use lazy_static::lazy_static;
use regex::Regex;
use scraper::{Html, Selector};
use serde::Serialize;
use std::{cell::Cell, fs, path::Path};

/// Crate-wide result alias for fallible operations on [`QcResult`].
pub type Result<T> = std::result::Result<T, QcResultError>;

/// Errors that can occur while parsing a QC result HTML document.
#[derive(Debug, thiserror::Error)]
pub enum QcResultError {
    /// The QC result file could not be read from disk.
    #[error("failed to read QC result file `{path}`: {source}")]
    Io {
        /// The path that was attempted to be read.
        path: String,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// The hard-coded CSS selector used to locate result blocks is invalid.
    #[error("invalid CSS selector `{selector}`: {message}")]
    Selector {
        /// The selector string that failed to parse.
        selector: &'static str,
        /// Description of the selector parse failure.
        message: String,
    },

    /// A statically compiled regular expression failed to compile.
    #[error("invalid regular expression: {0}")]
    Regex(#[from] regex::Error),
}

impl QcResultError {
    fn io<P: AsRef<Path>>(path: P, source: std::io::Error) -> Self {
        Self::Io {
            path: path.as_ref().display().to_string(),
            source,
        }
    }
}

lazy_static! {
    static ref SPACES: Regex = Regex::new(r"\s+").expect("valid `\\s+` pattern");
    static ref NUMBER_OF_VARIABLES_IN_COMMON: Regex =
        Regex::new(r"Number of Variables in Common:\s(\d+)\.")
            .expect("valid `Number of Variables in Common` pattern");
    static ref NUMBER_OF_VARIABLES_WITH_DIFFERING_ATTRIBUTES: Regex =
        Regex::new(r"Number of Variables with Differing Attributes:\s(\d+)\.")
            .expect("valid `Number of Variables with Differing Attributes` pattern");
    static ref LISTING_OF_COMMON_VARIABLES_WITH_DIFFERING_ATTRIBUTES_ROW: Regex =
        Regex::new(r"(([0-9A-Za-z_]+)\s+)?([0-9A-Za-z_.]+)\s+([A-Za-z]+)\s+(\d+)\s+(.+)")
            .expect("valid listing-of-common-variables row pattern");
    static ref OBSERVATION_SUMMARY_LIST: Regex = Regex::new(r"([A-Za-z ]+)\s+(\d+)?\s+(\d+)")
        .expect("valid observation summary list pattern");
    static ref VARIABLES_WITH_UNEQUAL_VALUES: Regex =
        Regex::new(r"([A-Za-z_0-9]+)\s+([A-Z]+)\s+(\d+)\s+(\d+)\s+(.+)\s+(\d+)(\s+\d+)?")
            .expect("valid variables-with-unequal-values pattern");
    static ref VALUE_COMPARISON_RESULT_FOR_VARIABLES_HEADER: Regex =
        Regex::new(r"Obs\s+\|\|\s+([A-Za-z_0-9]+)").expect("valid value-comparison header pattern");
    static ref VALUE_COMPARISON_RESULT_FOR_VARIABLES_CELL: Regex =
        Regex::new(r"(\d+)\s+?\|\|\s+(.*)\s{2,}(.*)").expect("valid value-comparison cell pattern");
}

/// CSS selector that locates each `<pre class="batch">` block in the report.
const BATCH_SELECTOR: &str = "pre.batch";

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum ProcessStage {
    #[default]
    DataSetSummary,
    VariablesSummary,
    ListingOfCommonVariablesWithDifferingAttributes,
    ComparisonResultsForObservations,
    ObservationSummary,
    ValuesComparisonSummary,
    VariablesWithUnequalValues,
    ValueComparisonResultsForVariables,
    Unknown,
}

impl ProcessStage {
    pub fn convert_from_str<S: AsRef<str>>(source: S) -> ProcessStage {
        match source.as_ref().trim() {
            "Data Set Summary" => ProcessStage::DataSetSummary,
            "Variables Summary" => ProcessStage::VariablesSummary,
            "Listing of Common Variables with Differing Attributes" => {
                ProcessStage::ListingOfCommonVariablesWithDifferingAttributes
            }
            "Comparison Results for Observations" => ProcessStage::ComparisonResultsForObservations,
            "Observation Summary" => ProcessStage::ObservationSummary,
            "Values Comparison Summary" => ProcessStage::ValuesComparisonSummary,
            "Variables with Unequal Values" => ProcessStage::VariablesWithUnequalValues,
            "Value Comparison Results for Variables" => {
                ProcessStage::ValueComparisonResultsForVariables
            }
            _ => ProcessStage::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dataset {
    pub dataset: String,
    pub created: String,
    pub modified: String,
    pub nvar: String,
    pub nobs: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetSummmary {
    pub base: Option<Dataset>,
    pub compare: Option<Dataset>,
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VariablesSummary {
    pub number_of_variables_in_common: Option<String>,
    pub number_of_variables_with_differing_attributes: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableAttribute {
    pub dataset: String,
    pub variable_type: String,
    pub variable_length: String,
    pub label: String,
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableAttributeGroup {
    pub base: Option<VariableAttribute>,
    pub compare: Option<VariableAttribute>,
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableDifferAttributes {
    pub variable: String,
    pub attribute: VariableAttributeGroup,
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableDifferAttributesList {
    pub variables: Vec<VariableDifferAttributes>,
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservationSummaryList {
    pub observation: String,
    pub base: String,
    pub compare: String,
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservationSummary {
    pub summary_list: Vec<ObservationSummaryList>,
    pub summary_logs: Vec<String>,
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValuesComparsionSummary {
    pub number_of_variables_compared_with_all_observations_equal: Option<String>,
    pub number_of_variables_compared_with_some_observations_unequal: Option<String>,
    pub total_number_of_values_with_compare_unequal: Option<String>,
    pub total_number_of_values_not_exactly_equal: Option<String>,
    pub maximum_difference_criterion_value: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableWithUnequalValues {
    variable: String,
    variable_type: String,
    len1: String,
    len2: String,
    label: String,
    ndif: String,
    maxdif: String,
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueComparsionResultsForVariablesRow {
    obs: String,
    base: String,
    compare: String,
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueComparsionResultsForVariables {
    variable: String,
    records: Vec<ValueComparsionResultsForVariablesRow>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QcResult {
    pub dataset_summary: Option<DatasetSummmary>,
    pub variables_summary: Option<VariablesSummary>,
    pub list_of_common_variables_with_differing_attributes: Option<VariableDifferAttributesList>,
    pub comparsion_results_for_observations: Option<Vec<String>>,
    pub observation_summary: Option<ObservationSummary>,
    pub values_comparsion_summary: Option<ValuesComparsionSummary>,
    pub variable_with_unequal_values: Option<Vec<VariableWithUnequalValues>>,
    pub values_comparsion_results_for_variables: Option<Vec<ValueComparsionResultsForVariables>>,
}

#[derive(Debug, Default)]
pub struct QcResultHtmlParser {
    processing_stage: Cell<ProcessStage>,
}

impl QcResultHtmlParser {
    pub fn new() -> QcResultHtmlParser {
        QcResultHtmlParser::default()
    }

    /// Parse the QC result HTML document located at `filepath`.
    ///
    /// # Errors
    ///
    /// Returns [`QcResultError::Io`] if the file cannot be read, or
    /// [`QcResultError::Selector`] if the hard-coded CSS selector is invalid.
    pub fn parse<P: AsRef<Path>>(&self, filepath: P) -> Result<QcResult> {
        let mut result = QcResult::default();
        let document =
            fs::read_to_string(&filepath).map_err(|source| QcResultError::io(&filepath, source))?;
        let dom = Html::parse_document(&document);
        let selector =
            Selector::parse(BATCH_SELECTOR).map_err(|source| QcResultError::Selector {
                selector: BATCH_SELECTOR,
                message: source.to_string(),
            })?;
        for parts in dom.select(&selector) {
            for row in parts.inner_html().split("\n") {
                if !row.trim().is_empty() {
                    let s = ProcessStage::convert_from_str(&row);
                    if s.ne(&ProcessStage::Unknown) {
                        self.processing_stage.set(s);
                    }
                    match self.processing_stage.get() {
                        ProcessStage::DataSetSummary => {
                            self.process_dataset_summary(row, &mut result);
                        }
                        ProcessStage::VariablesSummary => {
                            self.process_variable_summary(row, &mut result);
                        }
                        ProcessStage::ListingOfCommonVariablesWithDifferingAttributes => self
                            .process_listing_of_common_variables_with_differing_attributes(
                                row,
                                &mut result,
                            ),
                        ProcessStage::ComparisonResultsForObservations => {
                            self.process_comparsion_results_for_observations(row, &mut result)
                        }
                        ProcessStage::ObservationSummary => {
                            self.process_observation_summary(row, &mut result)
                        }
                        ProcessStage::ValuesComparisonSummary => {
                            self.process_values_comparsion_summary(row, &mut result)
                        }
                        ProcessStage::VariablesWithUnequalValues => {
                            self.process_variables_with_unequal_values(row, &mut result)
                        }
                        ProcessStage::ValueComparisonResultsForVariables => {
                            self.process_value_comparsion_results_for_variables(row, &mut result)
                        }
                        ProcessStage::Unknown => (),
                    }
                }
            }
        }
        Ok(result)
    }

    fn process_dataset_summary(&self, row: &str, result: &mut QcResult) {
        let mut dataset_summary = result.dataset_summary.clone();
        let row = SPACES.replace_all(row.trim(), " ");
        let row = row.split(" ").collect::<Vec<&str>>();
        if row.len() == 5 {
            if let Ok(_) = row
                .get(3)
                .expect("row length checked above")
                .parse::<usize>()
            {
                let summary = Some(Dataset {
                    dataset: row.first().expect("row length checked above").to_string(),
                    created: row.get(1).expect("row length checked above").to_string(),
                    modified: row.get(2).expect("row length checked above").to_string(),
                    nvar: row.get(3).expect("row length checked above").to_string(),
                    nobs: row.get(4).expect("row length checked above").to_string(),
                });
                match &mut dataset_summary {
                    Some(dataset_summary) => dataset_summary.compare = summary,
                    None => {
                        dataset_summary = Some(DatasetSummmary {
                            base: summary,
                            compare: None,
                        })
                    }
                }
            }
        }
        result.dataset_summary = dataset_summary;
    }

    fn process_variable_summary(&self, row: &str, result: &mut QcResult) {
        let mut variable_summary = match result.variables_summary.clone() {
            Some(variables_summary) => variables_summary,
            None => VariablesSummary::default(),
        };
        if let Some(target) = NUMBER_OF_VARIABLES_IN_COMMON.captures(row) {
            let target = target.get(1).expect("capture group 1 exists").as_str();
            variable_summary.number_of_variables_in_common = Some(target.to_string());
        } else if let Some(target) = NUMBER_OF_VARIABLES_WITH_DIFFERING_ATTRIBUTES.captures(row) {
            let target = target.get(1).expect("capture group 1 exists").as_str();
            variable_summary.number_of_variables_with_differing_attributes =
                Some(target.to_string());
        }
        result.variables_summary = Some(variable_summary)
    }

    fn process_listing_of_common_variables_with_differing_attributes(
        &self,
        row: &str,
        result: &mut QcResult,
    ) {
        if row
            .trim()
            .eq("Listing of Common Variables with Differing Attributes")
            || row
                .trim()
                .eq("Variable  Dataset                 Type  Length  Label")
        {
            return;
        }
        let mut list_of_common_variables_with_differing_attributes = match result
            .list_of_common_variables_with_differing_attributes
            .clone()
        {
            Some(list_of_common_variables_with_differing_attributes) => {
                list_of_common_variables_with_differing_attributes
            }
            None => VariableDifferAttributesList::default(),
        };
        if let Some(target) =
            LISTING_OF_COMMON_VARIABLES_WITH_DIFFERING_ATTRIBUTES_ROW.captures(row.trim())
        {
            let variable = target.get(2).map(|f| f.as_str().to_string());
            let dataset = target
                .get(3)
                .map(|f| f.as_str().to_string())
                .unwrap_or_default();
            let variable_type = target
                .get(4)
                .map(|f| f.as_str().to_string())
                .unwrap_or_default();
            let variable_length = target
                .get(5)
                .map(|f| f.as_str().to_string())
                .unwrap_or_default();
            let label = target
                .get(6)
                .map(|f| f.as_str().to_string())
                .unwrap_or_default();
            let variable_attr = VariableAttribute {
                dataset,
                variable_type,
                variable_length,
                label,
            };
            let variable = match variable {
                Some(name) => VariableDifferAttributes {
                    variable: name,
                    attribute: VariableAttributeGroup {
                        base: Some(variable_attr),
                        compare: None,
                    },
                },
                None => {
                    let mut variable = list_of_common_variables_with_differing_attributes
                        .variables
                        .pop()
                        .unwrap_or_default();
                    variable.attribute.compare = Some(variable_attr);
                    variable
                }
            };
            list_of_common_variables_with_differing_attributes
                .variables
                .push(variable);
        }
        result.list_of_common_variables_with_differing_attributes =
            Some(list_of_common_variables_with_differing_attributes);
    }

    fn process_comparsion_results_for_observations(&self, row: &str, result: &mut QcResult) {
        let row = row.trim();
        if row.eq("Comparison Results for Observations") {
            return;
        }

        let mut comparsion_results_for_observations =
            match result.comparsion_results_for_observations.clone() {
                Some(comparsion_results_for_observations) => comparsion_results_for_observations,
                None => vec![],
            };
        if row.starts_with("Observation") {
            comparsion_results_for_observations.push(row.to_string());
        }
        result.comparsion_results_for_observations = Some(comparsion_results_for_observations);
    }

    fn process_observation_summary(&self, row: &str, result: &mut QcResult) {
        let mut observation_summary = match result.observation_summary.clone() {
            Some(observation_summary) => observation_summary,
            None => ObservationSummary::default(),
        };
        let row = row.trim();
        if row.starts_with("Number of Observations")
            || row.starts_with("Total Number of Observations Read from")
        {
            observation_summary.summary_logs.push(row.to_string());
        }
        if let Some(target) = OBSERVATION_SUMMARY_LIST.captures(row) {
            let observation = target
                .get(1)
                .map(|s| s.as_str().trim().replace("  ", " ").to_string())
                .unwrap_or_default();
            let base = target
                .get(2)
                .map(|s| s.as_str().to_string())
                .unwrap_or_default();
            let compare = target
                .get(3)
                .map(|s| s.as_str().to_string())
                .unwrap_or_default();
            observation_summary
                .summary_list
                .push(ObservationSummaryList {
                    observation,
                    base,
                    compare,
                });
        }
        result.observation_summary = Some(observation_summary);
    }

    fn process_values_comparsion_summary(&self, row: &str, result: &mut QcResult) {
        let row = row.trim();
        if row.eq("Values Comparison Summary") {
            return;
        }

        let mut values_comparsion_summary = match result.values_comparsion_summary.clone() {
            Some(values_comparsion_summary) => values_comparsion_summary,
            None => ValuesComparsionSummary::default(),
        };
        let split = row.split(":").collect::<Vec<&str>>();
        if split.len().eq(&2) {
            let key = split.first().expect("split length checked above").trim();
            let value = split
                .get(1)
                .expect("split length checked above")
                .trim()
                .trim_end_matches('.');
            match key {
                "Number of Variables Compared with All Observations Equal" => {
                    values_comparsion_summary
                        .number_of_variables_compared_with_all_observations_equal =
                        Some(value.to_string())
                }
                "Number of Variables Compared with Some Observations Unequal" => {
                    values_comparsion_summary
                        .number_of_variables_compared_with_some_observations_unequal =
                        Some(value.to_string())
                }
                "Total Number of Values which Compare Unequal" => {
                    values_comparsion_summary.total_number_of_values_with_compare_unequal =
                        Some(value.to_string())
                }
                "Total Number of Values not EXACTLY Equal" => {
                    values_comparsion_summary.total_number_of_values_not_exactly_equal =
                        Some(value.to_string())
                }
                "Maximum Difference Criterion Value" => {
                    values_comparsion_summary.maximum_difference_criterion_value =
                        Some(value.to_string())
                }
                _ => (),
            }
        }
        result.values_comparsion_summary = Some(values_comparsion_summary);
    }

    fn process_variables_with_unequal_values(&self, row: &str, result: &mut QcResult) {
        let row = row.trim();
        if row.eq("Variables with Unequal Values")
            || row.eq("Variable  Type  Len1 Len2   Label                     Ndif   MaxDif ")
        {
            return;
        }
        let mut variable_with_unequal_values = match result.variable_with_unequal_values.clone() {
            Some(variable_with_unequal_values) => variable_with_unequal_values,
            None => vec![],
        };
        if let Some(target) = VARIABLES_WITH_UNEQUAL_VALUES.captures(row) {
            let variable = target
                .get(1)
                .map(|s| s.as_str().to_string())
                .unwrap_or_default();
            let variable_type = target
                .get(2)
                .map(|s| s.as_str().to_string())
                .unwrap_or_default();
            let len1 = target
                .get(3)
                .map(|s| s.as_str().to_string())
                .unwrap_or_default();
            let len2 = target
                .get(4)
                .map(|s| s.as_str().to_string())
                .unwrap_or_default();
            let label = target
                .get(5)
                .map(|s| s.as_str().trim().to_string())
                .unwrap_or_default();
            let ndif = target
                .get(6)
                .map(|s| s.as_str().to_string())
                .unwrap_or_default();
            let maxdif = target
                .get(7)
                .map(|s| s.as_str().trim().to_string())
                .unwrap_or_default();
            variable_with_unequal_values.push(VariableWithUnequalValues {
                variable,
                variable_type,
                len1,
                len2,
                label,
                ndif,
                maxdif,
            });
        }
        result.variable_with_unequal_values = Some(variable_with_unequal_values);
    }

    fn process_value_comparsion_results_for_variables(&self, row: &str, result: &mut QcResult) {
        let row = row.trim();
        let mut values_comparsion_results_for_variables =
            match result.values_comparsion_results_for_variables.clone() {
                Some(values_comparsion_results_for_variables) => {
                    values_comparsion_results_for_variables
                }
                None => vec![],
            };
        if let Some(target) = VALUE_COMPARISON_RESULT_FOR_VARIABLES_HEADER.captures(row) {
            let incoming_variable = target
                .get(1)
                .expect("capture group 1 exists")
                .as_str()
                .to_string();
            if let Some(last) = values_comparsion_results_for_variables.last() {
                if incoming_variable.ne(&last.variable) {
                    values_comparsion_results_for_variables.push(
                        ValueComparsionResultsForVariables {
                            variable: incoming_variable,
                            records: vec![],
                        },
                    );
                }
            } else {
                values_comparsion_results_for_variables.push(ValueComparsionResultsForVariables {
                    variable: incoming_variable,
                    records: vec![],
                });
            }
        } else if let Some(target) = VALUE_COMPARISON_RESULT_FOR_VARIABLES_CELL.captures(row) {
            let obs = target
                .get(1)
                .map(|s| s.as_str().to_string())
                .unwrap_or_default();
            let base = target
                .get(2)
                .map(|s| s.as_str().trim().to_string())
                .unwrap_or_default();
            let compare = target
                .get(3)
                .map(|s| s.as_str().trim().to_string())
                .unwrap_or_default();
            let record = ValueComparsionResultsForVariablesRow { obs, base, compare };
            if let Some(variable) = values_comparsion_results_for_variables.last_mut() {
                variable.records.push(record);
            }
        }
        result.values_comparsion_results_for_variables =
            Some(values_comparsion_results_for_variables);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_qc_result_parse() {
        let html = r#"<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01 Transitional//EN">
<html>

<head>
  <meta name="Generator" content="SAS Software Version 9.4, see www.sas.com">
  <meta http-equiv="Content-type" content="text/html; charset=utf-8">
  <title>SAS Output</title>
</head>

<body class="body">
  <div class="branch">
    <a name="IDX"></a>
    <table class="systitleandfootercontainer" width="100%" cellspacing="1" cellpadding="1" rules="none" frame="void"
      border="0" summary="Page Layout">
      <tr>
        <td class="c systemtitle">Compare Result for DatasSet LB</td>
      </tr>
    </table><br>
    <div>
      <div align="center">
        <table cellspacing="0" cellpadding="0" summary="Page Layout">
          <tr>
            <td>
              <pre class="batch">
                                                                                                                     The COMPARE Procedure
                                                                                                     Comparison of WORK._COMP_BASE with WORK._COMP_COMPARE
                                                                                                             (Method=ABSOLUTE, Criterion=0.000001)


                                                                                                                       Data Set Summary


                                                                                              Dataset                      Created          Modified  NVar    NObs

                                                                                              WORK._COMP_BASE     08MAY26:15:55:36  08MAY26:15:55:36    30   19382
                                                                                              WORK._COMP_COMPARE  08MAY26:15:55:36  08MAY26:15:55:36    30   19382



                                                                                                                       Variables Summary

                                                                                                             Number of Variables in Common: 30.
</pre>
            </td>
          </tr>
        </table>
      </div>
    </div>
    <br>
    <a name="IDX1"></a>
    <div>
      <div align="center">
        <table cellspacing="0" cellpadding="0" summary="Page Layout">
          <tr>
            <td>
              <pre class="batch">

                                                                                                                      Observation Summary


                                                                                                                 Observation      Base  Compare

                                                                                                                 First Obs           1        1
                                                                                                                 Last  Obs       19382    19382

                                                                                               Number of Observations in Common: 19382.
                                                                                               Total Number of Observations Read from WORK._COMP_BASE: 19382.
                                                                                               Total Number of Observations Read from WORK._COMP_COMPARE: 19382.

                                                                                               Number of Observations with Some Compared Variables Unequal: 0.
                                                                                               Number of Observations with All Compared Variables Equal: 19382.

                                                                                               NOTE: No unequal values were found. All values compared are exactly equal.

</pre>
            </td>
          </tr>
        </table>
      </div>
    </div>
    <br>
  </div>
</body>

</html>"#;

        let mut temp_file = NamedTempFile::new().expect("failed to create temp file");
        temp_file
            .write_all(html.as_bytes())
            .expect("failed to write QC result HTML");

        let parser = QcResultHtmlParser::new();
        let result = parser
            .parse(temp_file.path())
            .expect("failed to parse QC result HTML");

        // Dataset summary: the first row fills `base`, the second row fills `compare`.
        let dataset_summary = result.dataset_summary.expect("dataset_summary missing");
        let base = dataset_summary.base.expect("base dataset missing");
        assert_eq!(base.dataset, "WORK._COMP_BASE");
        assert_eq!(base.created, "08MAY26:15:55:36");
        assert_eq!(base.modified, "08MAY26:15:55:36");
        assert_eq!(base.nvar, "30");
        assert_eq!(base.nobs, "19382");

        let compare = dataset_summary.compare.expect("compare dataset missing");
        assert_eq!(compare.dataset, "WORK._COMP_COMPARE");
        assert_eq!(compare.nvar, "30");
        assert_eq!(compare.nobs, "19382");

        // Variables summary: a single "in common" count is present.
        let variables_summary = result.variables_summary.expect("variables_summary missing");
        assert_eq!(
            variables_summary.number_of_variables_in_common,
            Some("30".to_string())
        );

        // Observation summary: both First/Last rows and the summary log lines should be captured.
        let observation_summary = result
            .observation_summary
            .expect("observation_summary missing");
        assert!(
            observation_summary
                .summary_list
                .iter()
                .any(|r| r.observation == "First Obs" && r.base == "1" && r.compare == "1"),
            "expected `First Obs` row in summary_list, got {:?}",
            observation_summary.summary_list
        );
        assert!(
            observation_summary
                .summary_list
                .iter()
                .any(|r| { r.observation == "First Obs" && r.base == "1" && r.compare == "1" }),
            "expected `Last  Obs` row in summary_list, got {:?}",
            observation_summary.summary_list
        );
        assert!(
            observation_summary
                .summary_logs
                .iter()
                .any(|line| line.starts_with("Number of Observations in Common")),
            "expected `Number of Observations in Common` log line, got {:?}",
            observation_summary.summary_logs
        );
    }
}
