use anyhow::{Context, Result};
use ruff_python_ast::PySourceType;
use ruff_python_formatter::format_module_ast;
use ruff_python_parser::{parse, AsMode};
use ruff_python_trivia::CommentRanges;
use ruff_workspace::configuration::Configuration;
use ruff_workspace::resolver::{
    match_exclusion, resolve_root_settings, ConfigurationTransformer, Relativity,
};
use ruff_workspace::{pyproject, Settings};
use std::path::PathBuf;

struct DummyConfigurationTransformer {}

impl ConfigurationTransformer for DummyConfigurationTransformer {
    fn transform(&self, config: Configuration) -> Configuration {
        config
    }
}

pub enum FormatResult {
    Success { formatted_content: String },
    Ignored,
    Error { error: String },
}

pub fn format(given_file_name: String, file_content: String) -> Result<FormatResult> {
    let target_path = PathBuf::from(given_file_name);

    let source_type =
        PySourceType::from_extension(target_path.extension().unwrap().to_str().unwrap());

    let pyproject = pyproject::find_settings_toml(&target_path)?;
    let settings = match &pyproject {
        None => Settings::default(),
        Some(p) => resolve_root_settings(p, Relativity::Parent, &DummyConfigurationTransformer {})?,
    };

    let ignore = match_exclusion(
        &target_path,
        target_path.file_name().context("Failed to get file name")?,
        &settings.formatter.exclude,
    ) || match_exclusion(
        &target_path,
        target_path.file_name().context("Failed to get file name")?,
        &settings.file_resolver.exclude,
    );

    if ignore {
        return Ok(FormatResult::Ignored);
    }

    let options = settings
        .formatter
        .to_format_options(source_type, &file_content);

    let parsed = parse(&file_content, source_type.as_mode()).context("Syntax error in input")?;
    let comment_ranges = CommentRanges::from(parsed.tokens());
    let formatted = format_module_ast(&parsed, &comment_ranges, &file_content, options)
        .context("Failed to format node")?;

    Ok(FormatResult::Success {
        formatted_content: formatted.print()?.into_code(),
    })
}
