use syntect::{
    dumps::{dump_to_file, dump_to_uncompressed_file},
    highlighting::ThemeSet,
    parsing::SyntaxSetBuilder,
};

const THEME_OUTPUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/themes.themedump");
const THEMES_FOLDER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/themes/");

const SYNTAX_OUTPUT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/syntaxes.packdump");
const SYNTAXES_FOLDER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/syntaxes/");

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let mut syntax_builder = SyntaxSetBuilder::new();
    syntax_builder.add_plain_text_syntax();
    syntax_builder.add_from_folder(SYNTAXES_FOLDER, true)?;

    dump_to_uncompressed_file(&syntax_builder.build(), SYNTAX_OUTPUT)?;

    let themes = ThemeSet::load_from_folder(THEMES_FOLDER)?;
    dump_to_file(&themes, THEME_OUTPUT)?;

    Ok(())
}
