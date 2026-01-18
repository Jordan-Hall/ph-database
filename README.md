# Predator Hunters Criminal Convictions Database

A GDS-styled web application for journalists to track and manage criminal conviction records from court proceedings and interviews.

## Features

- **GDS Design System**: Professional UK Government Digital Service styling
- **Search & Filter**: Search records by name, offense type, city, and court
- **Interactive Map**: View conviction locations on an interactive map (street-level only)
- **Data Entry**: Add new records with comprehensive details
- **Consent Tracking**: Track interview consent and source information
- **Browser Storage**: Data persists in browser localStorage (can be upgraded to SurrealDB server)

## Technology Stack

- **Frontend**: Dioxus 0.7 (Rust web framework)
- **Storage**: Browser localStorage (structured for easy SurrealDB migration)
- **Mapping**: Leaflet.js
- **Design**: GDS (Government Digital Service) design system
- **Language**: Rust + WebAssembly

## Prerequisites

- Rust (latest stable version)
- Dioxus CLI: `cargo install dioxus-cli`
- Modern web browser

## Installation

1. **Clone the repository**
   ```bash
   cd ph-database
   ```

2. **Install dependencies**
   ```bash
   cargo build
   ```

3. **Run the development server**
   ```bash
   dx serve
   ```

4. **Open your browser**
   Navigate to `http://localhost:8080`

## Building for Production

```bash
dx build --release
```

The built files will be in `dist/` directory.

## Usage Guide

### Adding a New Record

1. Click **"Add New Record"** tab
2. Fill in the required fields:
   - **Personal Details**: Full name as recorded in court
   - **Conviction Details**: Offense type, date, court, sentence
   - **Location**: Street name (no house numbers), city, postcode district
   - **Coordinates**: Optional latitude/longitude for map display
   - **Source**: Court record, interview, or both
   - **Consent**: Check if interview consent was obtained

3. Click **"Save Record"**

### Searching Records

1. On the **"Search & View"** tab, enter search criteria:
   - Name (full or partial)
   - Offense type
   - City
   - Court name

2. Click **"Search"** or **"View All Records"**
3. Results appear in a table below
4. Click **"View"** to see full record details

### Map View

1. Click the **"Map View"** tab
2. Records with coordinates appear as markers
3. Click markers to see basic information
4. Map is centered on the UK

### Viewing a Record

1. From search results, click **"View"**
2. See all record details
3. View location on mini-map (if coordinates available)
4. Delete record if needed (warning button)

## Data Privacy & Legal Considerations

⚠️ **Important**: This tool is designed for journalism purposes with the following guidelines:

1. **Public Records Only**: Only record information from public court records
2. **Street-Level Only**: Do not record exact house numbers, only street names
3. **Consent Required**: For interviewed subjects, obtain and document consent
4. **Legal Compliance**: Ensure compliance with:
   - UK Data Protection Act 2018
   - GDPR
   - Rehabilitation of Offenders Act
   - Local privacy laws

5. **Ethical Use**: This tool is for legitimate journalism, not surveillance or harassment

## Data Storage

Currently using browser localStorage for data persistence:
- Data stored locally in your browser
- Not synchronized across devices
- Can be exported/imported (feature coming soon)

### Future: SurrealDB Integration

The codebase is structured to support SurrealDB server:
- Replace `database/mod.rs` with SurrealDB client
- Run SurrealDB server locally or remotely
- Enable multi-user access and better data management

## Project Structure

```
ph-database/
├── src/
│   ├── components/          # Reusable UI components
│   │   ├── header.rs        # GDS header
│   │   ├── form_components.rs # Form inputs
│   │   └── map.rs           # Interactive map
│   ├── database/            # Data storage layer
│   │   └── mod.rs           # localStorage abstraction
│   ├── models/              # Data models
│   │   └── mod.rs           # ConvictionRecord, SearchFilters
│   ├── pages/               # Application pages
│   │   ├── home.rs          # Search & view page
│   │   ├── add_record.rs    # Add record form
│   │   └── view_record.rs   # Record details
│   └── main.rs              # App entry point & routing
├── assets/
│   └── gds-styles.css       # GDS design system styles
├── index.html               # HTML entry point
├── Cargo.toml               # Rust dependencies
├── Dioxus.toml              # Dioxus configuration
└── README.md                # This file
```

## Development

### Running Tests
```bash
cargo test
```

### Code Style
```bash
cargo fmt
cargo clippy
```

### Watch Mode
The Dioxus CLI automatically watches for changes:
```bash
dx serve
```

## Roadmap

- [ ] Export/import data (CSV, JSON)
- [ ] SurrealDB server integration
- [ ] Advanced search filters (date ranges)
- [ ] Bulk import from court records
- [ ] PDF report generation
- [ ] Multi-user authentication
- [ ] Audit log for data changes
- [ ] Data backup and restore

## Contributing

This is a journalism tool. Contributions should maintain:
- Professional GDS design standards
- Data privacy compliance
- Ethical usage guidelines

## License

[Add your license here]

## Support

For issues or questions, contact the Predator Hunters journalism team.

## Disclaimer

This tool is designed for legitimate journalism purposes only. Users are responsible for:
- Ensuring legal compliance in their jurisdiction
- Obtaining necessary consents
- Using data ethically and responsibly
- Not using the tool for harassment, surveillance, or illegal purposes

The developers assume no liability for misuse of this software.
