pub(crate) fn list_to_string<I, S>(list: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    list.into_iter()
        .map(|a| a.as_ref().to_string())
        .collect::<Vec<String>>()
        .join(",")
}
