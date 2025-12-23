use anyhow::{Context, Result};
use clap::Parser;
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

// Grid layout constants
const GRID_COLS: usize = 4;
const GRID_ROWS: usize = 4;
const CARDS_PER_PAGE: usize = GRID_COLS * GRID_ROWS;

// A4 dimensions in mm
const A4_WIDTH_MM: f32 = 210.0;
const A4_HEIGHT_MM: f32 = 297.0;
const MARGIN_MM: f32 = 5.0;

// Calculate card dimensions
const CARD_WIDTH_MM: f32 = (A4_WIDTH_MM - 2.0 * MARGIN_MM) / GRID_COLS as f32;
const CARD_HEIGHT_MM: f32 = (A4_HEIGHT_MM - 2.0 * MARGIN_MM) / GRID_ROWS as f32;

// Text positioning
const TEXT_MARGIN_MM: f32 = 10.0;

#[derive(Parser, Debug)]
#[command(author, version, about = "Generate flash cards PDF from CSV", long_about = None)]
struct Args {
    /// Input CSV file (pipe-delimited, no headers)
    #[arg(short, long)]
    input: PathBuf,

    /// Output PDF file
    #[arg(short, long)]
    output: PathBuf,
}

#[derive(Debug, Clone)]
struct FlashCard {
    side_a: String,
    side_b: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Read and parse CSV
    let cards = read_csv(&args.input)?;
    println!("Loaded {} flash cards from CSV", cards.len());

    // Generate PDF
    generate_pdf(&cards, &args.output)?;
    println!("Generated PDF: {}", args.output.display());

    Ok(())
}

fn read_csv(path: &PathBuf) -> Result<Vec<FlashCard>> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'|')
        .has_headers(false)
        .from_path(path)
        .context("Failed to open CSV file")?;

    let mut cards = Vec::new();

    for result in reader.records() {
        let record = result.context("Failed to read CSV record")?;

        if record.len() < 2 {
            anyhow::bail!("CSV record must have at least 2 columns");
        }

        cards.push(FlashCard {
            side_a: record[0].to_string(),
            side_b: record[1].to_string(),
        });
    }

    Ok(cards)
}

fn generate_pdf(cards: &[FlashCard], output_path: &PathBuf) -> Result<()> {
    // Calculate total pages needed (2 pages per sheet: front + back)
    let total_sheets = (cards.len() + CARDS_PER_PAGE - 1) / CARDS_PER_PAGE;
    let total_pages = total_sheets * 2; // front and back

    // Ensure even number of pages (pairs of sheets)
    let total_pages = if total_pages % 2 == 0 {
        total_pages
    } else {
        total_pages + 2 // Add one more sheet (front + back)
    };

    let (doc, page1, layer1) =
        PdfDocument::new("Flash Cards", Mm(A4_WIDTH_MM), Mm(A4_HEIGHT_MM), "Layer 1");

    // Load a built-in bold font
    let font = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;

    let mut current_layer = layer1;
    let mut current_page = page1;

    // Process cards in chunks of CARDS_PER_PAGE
    for sheet_idx in 0..(total_pages / 2) {
        let start_idx = sheet_idx * CARDS_PER_PAGE;
        let end_idx = (start_idx + CARDS_PER_PAGE).min(cards.len());
        let sheet_cards: Vec<Option<&FlashCard>> =
            (start_idx..end_idx).map(|i| cards.get(i)).collect();

        // Create front page (side A)
        if sheet_idx > 0 {
            let (page, layer) = doc.add_page(Mm(A4_WIDTH_MM), Mm(A4_HEIGHT_MM), "Layer 1");
            current_page = page;
            current_layer = layer;
        }

        draw_card_grid(
            &doc,
            current_layer,
            current_page,
            &font,
            &sheet_cards,
            true, // front side (A)
            CARD_WIDTH_MM,
            CARD_HEIGHT_MM,
            MARGIN_MM,
        );

        // Create back page (side B)
        let (page, layer) = doc.add_page(Mm(A4_WIDTH_MM), Mm(A4_HEIGHT_MM), "Layer 1");
        current_page = page;
        current_layer = layer;

        draw_card_grid(
            &doc,
            current_layer,
            current_page,
            &font,
            &sheet_cards,
            false, // back side (B)
            CARD_WIDTH_MM,
            CARD_HEIGHT_MM,
            MARGIN_MM,
        );
    }

    // Save the PDF
    let file = File::create(output_path).context("Failed to create output file")?;
    let mut writer = BufWriter::new(file);
    doc.save(&mut writer).context("Failed to save PDF")?;

    Ok(())
}

// Wrap text to fit within available space
fn wrap_text(text: &str, font_size: f32, available_height_mm: f32) -> Vec<String> {
    // Approximate character width for Helvetica Bold (pt to mm: 1pt ≈ 0.3528mm)
    // Using 0.5 instead of 0.6 to allow more characters per line
    let avg_char_width_mm = font_size * 0.5 * 0.3528;

    // Calculate max characters per line
    let max_chars = (available_height_mm / avg_char_width_mm) as usize;

    if text.len() <= max_chars {
        return vec![text.to_string()];
    }

    // Split text into words and wrap
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in words {
        let test_line = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };

        if test_line.len() <= max_chars {
            current_line = test_line;
        } else {
            if !current_line.is_empty() {
                lines.push(current_line);
            }
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        vec![text.to_string()]
    } else {
        lines
    }
}

fn draw_card_grid(
    doc: &PdfDocumentReference,
    layer: PdfLayerIndex,
    page: PdfPageIndex,
    font: &IndirectFontRef,
    cards: &[Option<&FlashCard>],
    is_front: bool,
    card_width_mm: f32,
    card_height_mm: f32,
    margin_mm: f32,
) {
    let current_layer = doc.get_page(page).get_layer(layer);

    for (idx, card_opt) in cards.iter().enumerate() {
        if let Some(card) = card_opt {
            // Calculate position in grid
            let (col, row) = if is_front {
                // Front side: normal order (left to right, top to bottom)
                (idx % GRID_COLS, idx / GRID_COLS)
            } else {
                // Back side: horizontally mirrored for flip on long edge
                // When you flip along the long edge (vertical axis), columns reverse
                let original_col = idx % GRID_COLS;
                let original_row = idx / GRID_COLS;
                (GRID_COLS - 1 - original_col, original_row)
            };

            // Calculate position (origin is bottom-left in PDF)
            let x = margin_mm + col as f32 * card_width_mm;
            let y = A4_HEIGHT_MM - margin_mm - (row + 1) as f32 * card_height_mm;

            // Draw card border
            let points = vec![
                (Point::new(Mm(x), Mm(y)), false),
                (Point::new(Mm(x + card_width_mm), Mm(y)), false),
                (
                    Point::new(Mm(x + card_width_mm), Mm(y + card_height_mm)),
                    false,
                ),
                (Point::new(Mm(x), Mm(y + card_height_mm)), false),
            ];

            let line = Line {
                points,
                is_closed: true,
            };

            current_layer.add_line(line);

            // Draw text rotated 90 degrees clockwise with word wrapping
            let text = if is_front { &card.side_a } else { &card.side_b };

            let font_size = 18.0;
            let line_spacing_mm = 7.0; // Horizontal space between wrapped lines

            // Available space for text (rotated, so height becomes the constraint)
            let available_height = card_height_mm - 2.0 * TEXT_MARGIN_MM;

            // Wrap text if needed
            let lines = wrap_text(text, font_size, available_height);

            // Starting position (same as before, centered horizontally)
            let text_x = x + card_width_mm / 2.0;
            let text_y = y + card_height_mm - TEXT_MARGIN_MM;

            // Draw each line, offset horizontally for rotation
            for (i, line) in lines.iter().enumerate() {
                let line_x = text_x - i as f32 * line_spacing_mm;

                current_layer.begin_text_section();
                current_layer.set_font(font, font_size);
                current_layer.set_line_height(font_size);

                current_layer.set_text_matrix(TextMatrix::TranslateRotate(
                    Mm(line_x).into(),
                    Mm(text_y).into(),
                    -90.0,
                ));

                current_layer.write_text(line, font);
                current_layer.end_text_section();
            }
        }
    }
}
