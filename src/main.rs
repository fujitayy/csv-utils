mod config {
    use std::path::PathBuf;
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    #[structopt(rename_all = "kebab-case")]
    pub struct Opt {
        #[structopt(short = "e", long, raw(aliases = r#"&["regexp", "re"]"#))]
        pub regex: String,

        /// 1行目を処理対象にする
        #[structopt(long)]
        pub no_header: bool,

        #[structopt(short, long, parse(from_os_str))]
        pub file: Option<PathBuf>,
    }

    impl Opt {
        pub fn from_args() -> Opt {
            <Self as StructOpt>::from_args()
        }
    }
}

use itertools::Itertools;
use regex::Regex;
use std::fs;
use std::io::{self, BufReader};

fn main() -> Result<(), exitfailure::ExitFailure> {
    let opt = config::Opt::from_args();

    if let Some(path) = opt.file.as_ref() {
        go(&opt, BufReader::new(fs::File::open(path)?))?;
    } else {
        go(&opt, BufReader::new(io::stdin()))?;
    }

    Ok(())
}

fn go<R: io::Read + 'static>(opt: &config::Opt, r: R) -> Result<(), failure::Error> {
    let records = filter(
        reader(r),
        CsvRecordTester::new(!opt.no_header, Regex::new(&opt.regex)?),
    );

    for r in records {
        println!("{}", r?.2);
    }

    Ok(())
}

fn reader<R: io::Read + 'static>(
    reader: R,
) -> impl Iterator<Item = Result<(usize, csv::StringRecord, String), csv::Error>> {
    csv::ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .from_reader(reader)
        .into_records()
        .enumerate()
        .map(|(i, r)| {
            r.map(|r| {
                let l = r.iter().join(",");
                (i, r, l)
            })
        })
}

fn filter<'a, I: IntoIterator<Item = Result<(usize, csv::StringRecord, String), csv::Error>>>(
    iter: I,
    tester: CsvRecordTester,
) -> impl Iterator<Item = Result<(usize, csv::StringRecord, String), csv::Error>> {
    iter.into_iter().filter(move |r| {
        r.as_ref()
            .map(|(i, _, l)| tester.test(*i, &l))
            .unwrap_or(false)
    })
}

#[derive(Debug)]
struct CsvRecordTester {
    print_header: bool,
    re: Regex,
}

impl CsvRecordTester {
    pub fn new(print_header: bool, re: Regex) -> Self {
        CsvRecordTester { print_header, re }
    }

    pub fn test(&self, i: usize, r: &str) -> bool {
        (i == 0 && self.print_header) || self.re.is_match(r)
    }
}

#[cfg(test)]
mod test {
    use super::config::Opt;
    use super::*;
    use structopt::StructOpt;

    #[test]
    fn args_usage_normal_short() {
        let args = &["cu", "-e", "hoge"];
        let opt = Opt::from_iter(args.iter());
        let expected = "hoge".to_string();
        assert_eq!(expected, opt.regex);
    }

    #[test]
    fn args_usage_normal_long() {
        let args = &["cu", "--regex", "fuga"];
        let opt = Opt::from_iter(args.iter());
        let expected = "fuga".to_string();
        assert_eq!(expected, opt.regex);
    }

    #[test]
    fn args_usage_normal_aliases() {
        let args = &["cu", "--regexp", "😺"];
        let opt = Opt::from_iter(args.iter());
        let expected = "😺".to_string();
        assert_eq!(expected, opt.regex);

        let args = &["cu", "--re", "𠮷野家"];
        let opt = Opt::from_iter(args.iter());
        let expected = "𠮷野家".to_string();
        assert_eq!(expected, opt.regex);
    }
}
