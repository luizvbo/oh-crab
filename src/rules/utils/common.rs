use regex::Regex;

pub fn replace_argument(script: &str, from_: &str, to: &str) -> String {
    let re = Regex::new(&format!(r" {}$", regex::escape(from_))).unwrap();
    let replaced_in_the_end = re.replace(script, format!(" {}", to));

    if replaced_in_the_end.as_ref() != script {
        replaced_in_the_end.into_owned()
    } else {
        script.replace(&format!(" {} ", from_), &format!(" {} ", to))
    }
}
