use comfy_table::{
    ContentArrangement, Row, Table, modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL_CONDENSED,
};
use selfie::{progress_reporter::port::ProgressReporter, validation::ValidationIssue};

pub(crate) struct ValidationTableReporter {
    table: Table,
}

impl ValidationTableReporter {
    pub(crate) fn new() -> Self {
        Self {
            table: Table::new(),
        }
    }

    pub(crate) fn setup(&mut self, header: Vec<&'static str>) -> &mut Self {
        self.table
            .load_preset(UTF8_FULL_CONDENSED)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(header);

        self
    }

    pub(crate) fn add_validation_errors(
        &mut self,
        error_issues: &[&ValidationIssue],
        reporter: &impl ProgressReporter,
    ) -> &mut Self {
        for error in error_issues {
            self.table.add_row(vec![
                reporter.format_error(error.category().to_string()),
                error.field().to_string(),
                error.message().to_string(),
                error
                    .suggestion()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
            ]);
        }

        self
    }

    pub(crate) fn add_validation_warnings(
        &mut self,
        warning_issues: &[&ValidationIssue],
        reporter: &impl ProgressReporter,
    ) -> &mut Self {
        for warning in warning_issues {
            self.table.add_row(vec![
                reporter.format_warning(warning.category().to_string()),
                warning.field().to_string(),
                warning.message().to_string(),
                warning
                    .suggestion()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
            ]);
        }

        self
    }

    pub(crate) fn add_row<T: Into<Row>>(&mut self, row: T) -> &mut Self {
        self.table.add_row(row);
        self
    }

    pub(crate) fn print(&self) {
        eprintln!("{}", &self.table);
    }
}

pub(crate) struct PackageListTableReporter {
    table: Table,
}

impl PackageListTableReporter {
    pub(crate) fn new() -> Self {
        Self {
            table: Table::new(),
        }
    }

    pub(crate) fn setup(&mut self, header: Vec<&'static str>) -> &mut Self {
        self.table
            .load_preset(UTF8_FULL_CONDENSED)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(header);

        self
    }

    pub(crate) fn add_row<T: Into<Row>>(&mut self, row: T) -> &mut Self {
        self.table.add_row(row);
        self
    }

    pub(crate) fn print(&self) {
        println!("{}", &self.table);
    }
}
