#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum AnnotationType{
    Match,//regular search result
    SelectedMatch,//one currently selected if user hits enter they would end up at result
    Number,
    Keyword,
    KnownValue,
    Type,
    Char,
    LifeTimeSpecifier,
    Comment,
}