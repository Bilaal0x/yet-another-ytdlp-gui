use super::*;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct UrlIntake {
    pub(crate) urls: Vec<String>,
    pub(crate) invalid_lines: Vec<String>,
}

impl UrlIntake {
    pub(crate) fn can_analyze(&self) -> bool {
        !self.urls.is_empty() && self.invalid_lines.is_empty()
    }
}

pub(crate) fn parse_urls(input: &str) -> Vec<String> {
    parse_url_intake(input).urls
}

pub(crate) fn parse_url_intake(input: &str) -> UrlIntake {
    let mut urls = Vec::new();
    let mut invalid_lines = Vec::new();

    for raw_line in input.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let cells = split_list_cells(line);
        let mut found_url = false;
        for candidate in &cells {
            let candidate = normalize_url_candidate(&candidate);
            if candidate.is_empty() {
                continue;
            }

            if is_valid_url(&candidate) {
                urls.push(candidate);
                found_url = true;
            }
        }

        if !found_url && !is_header_row(&cells) {
            invalid_lines.push(line.to_string());
        }
    }

    UrlIntake {
        urls,
        invalid_lines,
    }
}

pub(crate) fn start_analysis(mut ctx: FetchContext) {
    let intake = parse_url_intake(&(ctx.url_text)());

    if intake.urls.is_empty() {
        ctx.last_error.set(Some(AppError::new(
            i18n::t("error_no_url_title"),
            i18n::t("error_no_url_message"),
            "",
        )));
        return;
    }

    if !intake.invalid_lines.is_empty() {
        ctx.last_error.set(Some(AppError::new(
            i18n::t("error_invalid_url_title"),
            i18n::t_with(
                "error_invalid_url_message",
                &[("count", intake.invalid_lines.len().to_string())],
            ),
            intake.invalid_lines.join("\n"),
        )));
        return;
    }

    ctx.analysis_cancel_token.with_mut(|token| *token += 1);
    let cancel_generation = (ctx.analysis_cancel_token)();
    ctx.analysis.set(None);
    ctx.last_error.set(None);
    ctx.busy.set(true);
    ctx.analysis_running.set(true);
    ctx.backend.send(BackendAction::Analyze {
        urls: intake.urls,
        settings: (ctx.settings)(),
        cancel_generation,
    });
}

pub(crate) fn cancel_analysis(mut ctx: FetchContext) {
    ctx.analysis_cancel_token.with_mut(|token| *token += 1);
    ctx.analysis_running.set(false);
    ctx.busy.set(false);
}

#[cfg(feature = "desktop")]
pub(crate) fn import_url_list(mut ctx: FetchContext) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter(i18n::t("text_files"), &["txt", "csv", "list"])
        .pick_file()
    {
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                ctx.url_text.set(content);
                ctx.last_error.set(None);
            }
            Err(error) => {
                ctx.last_error.set(Some(AppError::new(
                    i18n::t("import_failed"),
                    i18n::t_with(
                        "could_not_read_file",
                        &[("path", path.display().to_string())],
                    ),
                    error.to_string(),
                )));
                ctx.screen.set(Screen::Error);
            }
        }
    }
}

#[cfg(not(feature = "desktop"))]
pub(crate) fn import_url_list(_ctx: FetchContext) {}

fn split_list_cells(line: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    let mut quoted = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' if quoted && chars.peek() == Some(&'"') => {
                current.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' | ';' | '\t' if !quoted => {
                cells.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    cells.push(current.trim().to_string());
    cells
}

fn normalize_url_candidate(value: &str) -> String {
    let mut candidate = value.trim();

    if candidate.starts_with('<') && candidate.ends_with('>') && candidate.len() > 2 {
        candidate = &candidate[1..candidate.len() - 1];
    }

    candidate.trim().to_string()
}

fn is_valid_url(value: &str) -> bool {
    if value.chars().any(char::is_whitespace) {
        return false;
    }

    let lower = value.to_ascii_lowercase();
    let Some(rest) = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))
    else {
        return false;
    };

    let host = rest.split('/').next().unwrap_or_default();
    !host.is_empty() && host.contains('.')
}

fn is_header_row(cells: &[String]) -> bool {
    cells.iter().any(|cell| {
        matches!(
            cell.trim().to_ascii_lowercase().as_str(),
            "url" | "urls" | "link" | "links" | "source" | "source_url" | "webpage_url"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_one_url_per_line_and_ignores_comments() {
        let intake = parse_url_intake(
            r#"
            # playlist links
            https://example.com/watch?v=one

            https://youtu.be/two
            "#,
        );

        assert_eq!(
            intake.urls,
            vec![
                "https://example.com/watch?v=one".to_string(),
                "https://youtu.be/two".to_string()
            ]
        );
        assert!(intake.invalid_lines.is_empty());
    }

    #[test]
    fn extracts_urls_from_csv_cells() {
        let intake = parse_url_intake(
            r#"
            title,url
            "First","https://example.com/one"
            Second;https://example.com/two
            "#,
        );

        assert_eq!(
            intake.urls,
            vec![
                "https://example.com/one".to_string(),
                "https://example.com/two".to_string()
            ]
        );
        assert!(intake.invalid_lines.is_empty());
    }

    #[test]
    fn reports_lines_without_supported_urls() {
        let intake = parse_url_intake(
            r#"
            www.example.com/no-scheme
            notes only
            https://example.com/ok
            "#,
        );

        assert_eq!(intake.urls, vec!["https://example.com/ok".to_string()]);
        assert_eq!(
            intake.invalid_lines,
            vec![
                "www.example.com/no-scheme".to_string(),
                "notes only".to_string()
            ]
        );
    }
}
