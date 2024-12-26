use std::error::Error;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::str::FromStr;

use hickory_server::proto::rr::LowerName;

#[derive(Clone, Debug)]
pub struct DomainFilter {
    routed_domains: Box<[LowerName]>,
}

fn check_is_subzone(domain: &str, routed_domain: &str) -> bool {
    let domain: Vec<&str> = domain.split(".").filter(|part| !part.is_empty()).collect();
    let routed_domain: Vec<&str> = routed_domain
        .split(".")
        .filter(|part| !part.is_empty())
        .collect();
    routed_domain
        .iter()
        .rev()
        .zip(domain.iter().rev())
        .all(|(part_a, part_b)| part_a == part_b)
}

impl DomainFilter {
    pub fn from_file(filename: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(filename)?;
        let domains: Box<[LowerName]> = io::BufReader::new(file)
            .lines()
            .map_while(Result::ok)
            .filter(|line| !line.is_empty())
            .filter_map(|domain| LowerName::from_str(domain.as_str()).ok())
            .collect();
        Ok(DomainFilter {
            routed_domains: domains,
        })
    }

    /// Проверяет, находится ли этот домен в списке для маршрутизации
    pub fn check(&self, domain: &LowerName) -> bool {
        self.routed_domains.iter().any(|routed_domain| {
            check_is_subzone(
                domain.to_string().as_str(),
                routed_domain.to_string().as_str(),
            )
        })
    }
}
