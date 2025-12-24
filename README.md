# Flash Cards Generator

A Rust command-line application that generates printable flash cards from CSV data.

## Features

- Generates A4 PDF documents with 16 flash cards per double-sided page (4x4 grid)
- Text is rotated 90 degrees clockwise to maximize space for longer content
- Supports pipe-delimited CSV input with no headers
- Correctly aligns front and back sides for double-sided printing along the long edge
- Automatically generates an even number of pages
- Fills all pages to cover all items in the CSV

## Installation

### Build from source

```bash
cargo build --release
```

The compiled binary will be located at `target/release/flash-cards-generator`.

## Usage

```bash
flash-cards-generator -i <input.csv> -o <output.pdf>
```

### Options

- `-i, --input <FILE>`: Input CSV file (pipe-delimited, no headers)
- `-o, --output <FILE>`: Output PDF file

### Example

```bash
./target/release/flash-cards-generator -i sample_flashcards.csv -o flashcards.pdf
```

## CSV Format

The input CSV file should:

- Use pipe (`|`) as the delimiter
- Have no headers
- Contain two columns: Side A | Side B

Example:

```text
Hello|Bonjour
Goodbye|Au revoir
Please|S'il vous plaît
Thank you|Merci
```

## Printing Instructions

1. Print the generated PDF using double-sided printing
2. Select "Flip on long edge" (or "Long-edge binding") in your printer settings
3. This ensures that when you flip the page vertically, the backs align correctly with the fronts

## Layout

- Each A4 page contains 16 cards in a 4x4 grid
- Card dimensions are automatically calculated to fit the page
- The application ensures proper alignment when pages are printed double-sided
