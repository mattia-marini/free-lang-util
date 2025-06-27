use std::collections::HashMap;

use super::{
    grammar::Grammar,
    parse_structs::{Action, FirstFollowSet},
};

use base64::Engine as _;

#[derive(Clone, Debug)]
pub struct LatexFormatOutputFormatDescriptor {
    pub grammophone_link: bool,
    pub graphviz_link: bool,
    pub grammar_definition: bool,
    pub lr0_parsing_table: bool,
    pub slr1_parsing_table: bool,
    pub first_follow_set: bool,
}

impl Default for LatexFormatOutputFormatDescriptor {
    fn default() -> Self {
        Self::FULL
    }
}

impl LatexFormatOutputFormatDescriptor {
    pub const FULL: Self = Self {
        grammophone_link: true,
        graphviz_link: true,
        grammar_definition: true,
        lr0_parsing_table: true,
        slr1_parsing_table: true,
        first_follow_set: true,
    };

    pub const NO_LINKS: Self = Self {
        grammophone_link: false,
        graphviz_link: false,
        grammar_definition: true,
        lr0_parsing_table: true,
        slr1_parsing_table: true,
        first_follow_set: true,
    };
}

impl Grammar {
    fn generate_grammophone_link(&self, sorted_non_terms: &Vec<char>) -> String {
        let mut grammar_str = String::new();

        let mut productions_by_driver = HashMap::new();
        for prod in self.productions.iter() {
            if !productions_by_driver.contains_key(&prod.driver) {
                productions_by_driver.insert(prod.driver, vec![]);
            }
            productions_by_driver
                .get_mut(&prod.driver)
                .unwrap()
                .push(prod);
        }
        for driver in sorted_non_terms.iter() {
            let mut bodies: Vec<String> = vec![];
            for prod in productions_by_driver.get(driver).unwrap().iter() {
                bodies.push(
                    prod.body
                        .iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<String>>()
                        .join(" "),
                );
            }
            grammar_str.push_str(format!("{} -> {} .\n", driver, bodies.join(" | ")).as_str());
        }

        format!(
            "https://mdaines.github.io/grammophone/?s={}",
            base64::engine::general_purpose::STANDARD.encode(grammar_str)
        )
    }

    fn generate_parsing_table_latex(
        &self,
        parsing_table: &Vec<HashMap<char, Vec<Action>>>,
        sorted_terms: &Vec<char>,
        sorted_non_terms: &Vec<char>,
        caption: Option<&str>,
    ) -> String {
        let mut rv = String::new();

        rv.push_str("\\begin{table}[H]");
        rv.push_str("\\centering");
        rv.push_str(
            format!(
                "\\begin{{tabular}}{{{}}}\n",
                "c".repeat(sorted_terms.len() + sorted_non_terms.len() + 1)
            )
            .as_str(),
        );
        rv.push_str("\\toprule\n");
        let header = format!(
            "States & {} & {}\\\\\n",
            sorted_terms
                .iter()
                .map(|c| if *c == '$' {
                    "\\$".to_string()
                } else {
                    c.to_string()
                })
                .collect::<Vec<String>>()
                .join(" & "),
            sorted_non_terms
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>()
                .join(" & ")
        );
        rv.push_str(header.as_str());
        rv.push_str("\\midrule\n");

        for (node_index, row) in parsing_table.iter().enumerate() {
            let mut row_str = vec![];
            for term in sorted_terms.iter() {
                if let Some(actions) = row.get(term) {
                    let actions_str: Vec<String> =
                        actions.iter().map(|action| action.to_string()).collect();
                    // actions_str = actions_str.join(", ");
                    // join("/");
                    row_str.push(actions_str.join("/"));
                } else {
                    row_str.push(String::from(" "));
                }
            }

            for non_term in sorted_non_terms.iter() {
                if let Some(actions) = row.get(non_term) {
                    let actions_str: Vec<String> =
                        actions.iter().map(|action| action.to_string()).collect();
                    // actions_str = actions_str.join(", ");
                    // join("/");
                    row_str.push(actions_str.join("/"));
                } else {
                    row_str.push(String::from(" "));
                }
            }

            rv.push_str(format!("s{} & {} \\\\ \n", node_index, row_str.join(" & ")).as_str());
        }

        rv.push_str("\\bottomrule\n");
        rv.push_str("\\end{tabular}\n");
        if let Some(caption) = caption {
            rv.push_str(format!("\\caption{{{}}}", caption).as_str());
        }
        rv.push_str("\\end{table}");
        rv
    }

    fn generate_first_follow_table_latex(
        &self,
        first_follow_set: Option<&HashMap<char, FirstFollowSet>>,
        sorted_terms: &Vec<char>,
        sorted_non_terms: &Vec<char>,
    ) -> String {
        let mut rv = String::new();

        let first_follow_set_owned;
        let first_follow_set = match first_follow_set {
            Some(s) => s,
            None => {
                first_follow_set_owned = self.get_first_follow_table();
                &first_follow_set_owned
            }
        };

        rv.push_str("\\begin{table}[H]");
        rv.push_str("\\centering");
        rv.push_str("\\begin{tabular}{cccc}\n");
        rv.push_str("\\toprule\n");
        rv.push_str("Symbol & First\\-set & Follow\\-set & Nullable\\\\\n");
        rv.push_str("\\midrule\n");
        for non_term in sorted_non_terms.iter() {
            if let Some(set) = first_follow_set.get(non_term) {
                let espace_latex_chars = |e: char| {
                    if e == '$' {
                        "\\$".to_string()
                    } else {
                        e.to_string()
                    }
                };

                let mut first_set = vec![];
                for c in sorted_terms.iter() {
                    if let Some(first) = set.first.get(c) {
                        first_set.push(*first);
                    }
                }

                let mut follow_set = vec![];
                for c in sorted_terms.iter() {
                    if let Some(follow) = set.follow.get(c) {
                        follow_set.push(*follow);
                    }
                }

                let first_set_str = first_set
                    .iter()
                    .cloned()
                    .map(espace_latex_chars)
                    .collect::<Vec<String>>()
                    .join(",");
                let follow_set_str: String = follow_set
                    .iter()
                    .cloned()
                    .map(espace_latex_chars)
                    .collect::<Vec<String>>()
                    .join(",");

                let nullable_str = if set.nullable { "Yes" } else { "No" };
                rv.push_str(
                    format!(
                        "{} & {} & {} & {}\\\\\n",
                        non_term, first_set_str, follow_set_str, nullable_str
                    )
                    .as_str(),
                );
            }
        }
        rv.push_str("\\bottomrule\n");
        rv.push_str("\\end{tabular}\n");
        rv.push_str("\\end{table}");
        rv
    }

    pub fn generate_latex_string(&self, descriptor: LatexFormatOutputFormatDescriptor) -> String {
        /* ######################### Common ######################### */
        let first_follow_set = self.get_first_follow_table();
        let lr0_parsing_table = self.get_lr0_parsing_table();
        let slr1_parsing_table =
            self.get_slr1_parsing_table(Some(&lr0_parsing_table), Some(&first_follow_set));

        let mut sorted_terms: Vec<char> = self.terms.iter().cloned().collect();
        sorted_terms.sort();
        sorted_terms.push('$');

        let mut sorted_non_terms: Vec<char> =
            self.productions.iter().map(|prod| prod.driver).collect();
        sorted_non_terms.dedup();

        /* ######################### Grammophone link ######################### */
        let mut grammophone_link_string = String::new();
        if descriptor.grammophone_link {
            grammophone_link_string = format!(
                "\\href{{{}}}{{View on Grammophone}}",
                self.generate_grammophone_link(&sorted_non_terms)
            )
        }

        /* ######################### Grammar ######################### */
        let mut productions_string = String::new();
        if descriptor.grammar_definition {
            productions_string.push_str("\\begin{align*}\n");
            let mut productions_by_driver = HashMap::new();
            for prod in self.productions.iter() {
                if !productions_by_driver.contains_key(&prod.driver) {
                    productions_by_driver.insert(prod.driver, vec![]);
                }
                productions_by_driver
                    .get_mut(&prod.driver)
                    .unwrap()
                    .push(prod);
            }
            for driver in sorted_non_terms.iter() {
                let mut bodies: Vec<String> = vec![];
                for prod in productions_by_driver.get(driver).unwrap().iter() {
                    let formatted_body = if prod.body.is_empty() {
                        "\\epsilon".to_string()
                    } else {
                        prod.body.iter().collect()
                    };
                    bodies.push(formatted_body);
                }
                productions_string.push_str(
                    format!("{} &\\rightarrow {} \\\\\n", driver, bodies.join(" \\mid ")).as_str(),
                );
            }
            productions_string.push_str("\\end{align*}\n");
        }
        let mut lr0_parsing_table_string = String::new();
        if descriptor.lr0_parsing_table {
            /* ######################### lr0 Parsing table ######################### */
            lr0_parsing_table_string = Self::generate_parsing_table_latex(
                self,
                &lr0_parsing_table,
                &sorted_terms,
                &sorted_non_terms,
                Some("Tabella di parsing LR(0)"),
            );
        }

        /* ######################### slr1 Parsing table ######################### */

        let mut slr1_parsing_table_string = String::new();
        if descriptor.slr1_parsing_table {
            slr1_parsing_table_string = Self::generate_parsing_table_latex(
                self,
                &slr1_parsing_table,
                &sorted_terms,
                &sorted_non_terms,
                Some("Tabella di parsing SLR(1)"),
            );
        }

        /* ######################### First follow table ######################### */
        let mut first_follow_table_string = String::new();
        if descriptor.first_follow_set {
            first_follow_table_string = Self::generate_first_follow_table_latex(
                self,
                Some(&first_follow_set),
                &sorted_terms,
                &sorted_non_terms,
            );
        }

        format!(
            "
% Grammophone link\n{} \n\n
% Grammar\n{} \n\n
% Lr0 parsing table\n{} \n\n
% Slr1 parsing table\n{} \n\n
% First-follow set\n{}
",
            grammophone_link_string,
            productions_string,
            lr0_parsing_table_string,
            slr1_parsing_table_string,
            first_follow_table_string
        )
    }
}
