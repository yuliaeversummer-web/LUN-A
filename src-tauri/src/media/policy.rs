// Observed with Windows GSMTC on the installed desktop application, not inferred
// from track/title text. See output/stage-5-discovery.json and docs/stage-5.md.
pub const YANDEX_DESKTOP_ID: &str = "ru.yandex.desktop.music";
pub fn eligible(source: &str) -> bool { source == YANDEX_DESKTOP_ID }
pub fn choose(candidates: &[String], manual: bool, retained: Option<&str>, current: Option<&str>) -> Option<String> {
 if manual {
  if let Some(retained)=retained.filter(|id|candidates.iter().any(|c|c==id)) {return Some(retained.into());}
  return (candidates.len()==1).then(||candidates[0].clone());
 }
 if candidates.len()==1 { return Some(candidates[0].clone()); }
 current.filter(|id|candidates.iter().any(|c|c==id)).map(str::to_owned)
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn never_matches_browser_or_similar_source() {
  assert!(eligible(YANDEX_DESKTOP_ID));
  for source in ["chrome.exe","YandexBrowser","Spotify.exe","ru.yandex.desktop.music.browser","https://music.yandex.ru",""] {assert!(!eligible(source));}
 }
 #[test] fn selection_is_explicit_and_does_not_fall_back_to_foreign_current() {
  let ids=vec!["a".into(),"b".into()];
  assert_eq!(choose(&ids,false,None,Some("a")),Some("a".into()));
  assert_eq!(choose(&ids,false,None,Some("browser")),None);
  assert_eq!(choose(&ids,true,Some("b"),Some("a")),Some("b".into()));
  assert_eq!(choose(&ids,true,Some("gone"),Some("a")),None);
  assert_eq!(choose(&["a".into()],false,None,Some("browser")),Some("a".into()));
 }
}
