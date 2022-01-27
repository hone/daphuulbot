use lazy_static::lazy_static;
use regex::Regex;
use select::{
    document::Document,
    predicate::{And, Attr, Name},
};
use std::io::Cursor;

// masquerade as a browser so kickstarter doesn't think we're an automation tool
static APP_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 6.3; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/59.0.3071.115 Safari/537.36";

#[derive(thiserror::Error, Debug)]
pub enum EmbedInfoError {
    #[error("Is not a Kickstarter link")]
    NotKickstarter,
    #[error("Problem scraping HTML")]
    EmptyDoc,
    #[error("Reqwest Error")]
    Reqwest(#[from] reqwest::Error),
    #[error("I/O Error")]
    Io(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct EmbedInfo {
    pub title: String,
    pub description: String,
    pub url: String,
    pub image: String,
}
use tracing::instrument;

impl EmbedInfo {
    #[instrument]
    /// Build a Discord Embed struct from a Kickstarter URL
    pub async fn from_url(link: &str) -> Result<Self, EmbedInfoError> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^https://www.kickstarter.com").unwrap();
        }
        if !RE.is_match(link) {
            return Err(EmbedInfoError::NotKickstarter);
        }
        let http = reqwest::Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()?;
        let html_bytes = http.get(link).send().await?.bytes().await?;
        let html_doc = Document::from_read(Cursor::new(&html_bytes))?;
        let embed_info = embed_info(&html_doc);

        if embed_info.is_empty() {
            return Err(EmbedInfoError::EmptyDoc);
        }

        Ok(embed_info)
    }

    /// If the EmbedInfo is empty
    pub fn is_empty(&self) -> bool {
        self.title.is_empty()
            && self.description.is_empty()
            && self.url.is_empty()
            && self.image.is_empty()
    }
}

#[instrument]
fn embed_info(document: &Document) -> EmbedInfo {
    EmbedInfo {
        title: find_property(document, "og:title"),
        description: find_property(document, "og:description"),
        url: find_property(document, "og:url"),
        image: find_property(document, "og:image"),
    }
}

#[instrument]
fn find_property(document: &Document, name: &str) -> String {
    let meta = And(Name("meta"), Attr("content", ()));
    document
        .find(And(meta, Attr("property", name)))
        .nth(0)
        .map(|node| node.attr("content").unwrap_or_default())
        .unwrap_or_default()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_errors_for_non_kickstarter_from_url() {
        let info = tokio_test::block_on(EmbedInfo::from_url("https://gamefound.com/projects/chip-theory-games/too-many-bones-unbreakable?previewPhase=CrowdfundingEnded"));

        let err = info.unwrap_err();
        assert!(matches!(err, EmbedInfoError::NotKickstarter));
    }

    #[test]
    fn it_errors_for_empty_doc() {}

    #[test]
    fn it_finds_embed_info() {
        let document = Document::from(include_str!(
            "../../fixtures/kickstarter/union-city-alliance.html"
        ));
        let info = embed_info(&document);
        assert_eq!(
            "Union City Alliance Heroes Unite Deckbuilding Board Game NEW",
            info.title
        );
        assert_eq!("Cooperative Superhero Deck and Board Building Game for 2 to 4 Players or Solo Gaming: RELAUNCH", info.description);
        assert_eq!("https://www.kickstarter.com/projects/paulmalchow/union-city-alliance-heroes-unite-deckbuilding-board-game-new", info.url);
        assert_eq!("https://ksr-ugc.imgix.net/assets/032/527/738/f6f807eea440d04d9e0a23c99a53d59f_original.jpg?ixlib=rb-2.1.0&crop=faces&w=1552&h=873&fit=crop&v=1614352887&auto=format&frame=1&q=92&s=9826cc4438e49cf65594eba473ca0af8", info.image);
    }

    #[test]
    fn it_fetches_from_kickstarter() {
        let info = tokio_test::block_on(EmbedInfo::from_url("https://www.kickstarter.com/projects/paulmalchow/union-city-alliance-heroes-unite-deckbuilding-board-game-new")).unwrap();
        assert_eq!(
            "Union City Alliance Heroes Unite Deckbuilding Board Game NEW",
            info.title
        );
        assert_eq!("Cooperative Superhero Deck and Board Building Game for 2 to 4 Players or Solo Gaming: RELAUNCH", info.description);
        assert_eq!("https://www.kickstarter.com/projects/paulmalchow/union-city-alliance-heroes-unite-deckbuilding-board-game-new", info.url);
    }

    #[test]
    fn it_errors_if_not_kickstarter() {
        let info = tokio_test::block_on(EmbedInfo::from_url(
            "https://gamefound.com/projects/lucky-duck-games/kingdom-rush-elemental-uprising/",
        ));

        assert!(info.is_err());
    }

    #[test]
    fn it_is_empty() {
        let info = EmbedInfo {
            title: "".to_string(),
            description: "".to_string(),
            url: "".to_string(),
            image: "".to_string(),
        };

        assert_eq!(info.is_empty(), true);
    }

    #[test]
    fn it_is_not_empty() {
        let info = EmbedInfo {
            title: "foo".to_string(),
            description: "".to_string(),
            url: "".to_string(),
            image: "".to_string(),
        };

        assert_eq!(info.is_empty(), false);
    }
}
