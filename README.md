# Genshin Package Scanner

A Rust-based GUI application that fetches and displays the latest game and audio package data for Genshin Impact. The application allows users to retrieve, format, and copy package information for easy sharing and downloading.

## Features

- **Fetch Latest Data**: Retrieves game and audio package information from the Genshin Impact API.
- **Separate "Copy" Buttons for Data Sections**: Allows users to copy specific sections of data independently:
  - **Main Data**
  - **Pre-download (Main)**
  - **Pre-download (Patches)**
- **"Clear" Button Implementation**: Resets all displayed data and error messages with a single click, providing a clean slate for new data fetches.
- **Data Structuring Enhancements**:
  - **Pre-download Data Split**: Divides Pre-download data into **Main** and **Patches** sections for better organization and readability.
  - **Version Formatting in Patches**: Displays version transitions clearly (e.g., `Version: 4.8.0 to 5.1.0`) with headings per version for improved clarity.
- **Language Name Mapping**: Replaces technical language codes with their full, understandable names:
  - `zh-cn` → `Chinese`
  - `en-us` → `English`
  - `ja-jp` → `Japanese`
  - `ko-kr` → `Korean`
- **Collapsed "Main Data" Section by Default**: The "Main Data" section remains collapsed after fetching data, allowing users to expand it manually as needed.
- **Cross-Platform Support**: Runs on Windows and Linux (GUI support required).
- **GUI Built with `eframe` and `egui`**: Provides a simple and intuitive user interface.
- **Robust Error Handling and Logging**: Displays error messages prominently and logs detailed information for troubleshooting.

## Downloads

You can download the latest compiled binaries from the [Releases](https://github.com/Zellith/genshin-api-scanner/releases) page.

## Usage

1. **Fetch Data**:
   - Click the **"Fetch Data"** button to retrieve the latest game and audio package information.
   - The application will automatically format and display the data in organized sections.
   
2. **Copy Data**:
   - Each data section (**Main Data**, **Pre-download (Main)**, **Pre-download (Patches)**) has its own **"Copy"** button.
   - Click the respective **"Copy"** button to copy the desired section to your clipboard.
   
3. **Clear Data**:
   - Click the **"Clear"** button to reset all displayed data and error messages, allowing you to start fresh.
   
4. **View Raw Data**:
   - Expand the **"Raw Main Data"** and **"Raw Pre-download Data"** sections to view the exact JSON responses fetched from the API. This is useful for debugging and verifying data integrity.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any features, bugs, or improvements.

## Contact

For any inquiries or support, please open an issue on the [GitHub repository](https://github.com/Zellith/genshin-api-scanner/issues).

---
