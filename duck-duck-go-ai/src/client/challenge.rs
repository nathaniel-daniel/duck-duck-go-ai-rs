use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use ego_tree::NodeId;
use ego_tree::NodeMut;
use ego_tree::NodeRef;
use markup5ever::namespace_url;
use markup5ever::ns;
use markup5ever::LocalName;
use markup5ever::QualName;
use rquickjs::class::Trace;
use rquickjs::class::Tracer;
use rquickjs::prelude::Func;
use rquickjs::runtime::UserDataError;
use rquickjs::runtime::UserDataGuard;
use rquickjs::CatchResultExt;
use rquickjs::Class;
use rquickjs::Context;
use rquickjs::Ctx;
use rquickjs::Exception;
use rquickjs::JsLifetime;
use rquickjs::Object;
use rquickjs::Promise;
use rquickjs::Runtime;
use scraper::node::Element;
use scraper::ElementRef;
use scraper::Html;
use scraper::Selector;
use sha2::Digest;
use sha2::Sha256;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Write;

#[derive(Debug, thiserror::Error)]
pub enum SolveError {
    #[error("js error: {0}")]
    Js(String),

    #[error("base64 decode error")]
    Bse64Decode(#[from] base64::DecodeError),

    #[error("invalid unicode")]
    InvalidUnicode(#[from] std::str::Utf8Error),

    #[error("json error")]
    Json(#[from] serde_json::Error),
}

impl From<rquickjs::Error> for SolveError {
    fn from(error: rquickjs::Error) -> Self {
        Self::Js(error.to_string())
    }
}

impl From<rquickjs::CaughtError<'_>> for SolveError {
    fn from(error: rquickjs::CaughtError) -> Self {
        Self::Js(error.to_string())
    }
}

impl From<UserDataError<UserData>> for SolveError {
    fn from(error: UserDataError<UserData>) -> Self {
        Self::Js(error.to_string())
    }
}

#[derive(JsLifetime)]
struct UserData {
    html: RefCell<Html>,
    user_agent: String,
}

fn get_user_data<'js, 'a>(ctx: &'a Ctx<'js>) -> rquickjs::Result<UserDataGuard<'a, UserData>> {
    match ctx.userdata::<UserData>() {
        Some(user_data) => Ok(user_data),
        None => Err(Exception::throw_message(ctx, "missing user data")),
    }
}

#[derive(JsLifetime)]
#[rquickjs::class]
struct NativeElement {
    id: NodeId,
}

#[rquickjs::methods]
impl NativeElement {
    #[qjs(get, rename = "innerHTML")]
    pub fn get_inner_html<'js>(&self, ctx: Ctx<'js>) -> rquickjs::Result<String> {
        let user_data = get_user_data(&ctx)?;
        let document = user_data.html.borrow();
        let element = match document.tree.get(self.id) {
            Some(el) => el,
            None => return Err(Exception::throw_message(&ctx, "missing node id")),
        };
        let element = match ElementRef::wrap(element) {
            Some(value) => value,
            None => return Err(Exception::throw_message(&ctx, "node is not an element")),
        };

        Ok(element.inner_html())
    }

    #[qjs(set, rename = "innerHTML")]
    pub fn set_inner_html<'js>(&mut self, ctx: Ctx<'js>, value: String) -> rquickjs::Result<()> {
        let user_data = get_user_data(&ctx)?;
        let mut document = user_data.html.borrow_mut();
        let document_element = match document.tree.get_mut(self.id) {
            Some(el) => el,
            None => return Err(Exception::throw_message(&ctx, "missing node id")),
        };

        fn copy_tree_recurse<T>(src: NodeRef<T>, mut dest: NodeMut<T>)
        where
            T: Clone + std::fmt::Debug,
        {
            for src_child in src.children() {
                let dest_child = dest.append(src_child.value().clone());
                copy_tree_recurse(src_child, dest_child);
            }
        }

        let fragment = Html::parse_fragment(&value);
        let src_html = match fragment.tree.root().children().next() {
            Some(value) => value,
            None => return Err(Exception::throw_message(&ctx, "missing html node")),
        };
        copy_tree_recurse(src_html, document_element);

        Ok(())
    }

    #[qjs(rename = "querySelectorAll")]
    pub fn query_selector_all<'js>(
        &mut self,
        ctx: Ctx<'js>,
        selector: String,
    ) -> rquickjs::Result<Vec<Class<'js, NativeElement>>> {
        let user_data = get_user_data(&ctx)?;
        let document = user_data.html.borrow();
        let element = match document.tree.get(self.id) {
            Some(el) => el,
            None => return Err(Exception::throw_message(&ctx, "missing node id")),
        };
        let element = match ElementRef::wrap(element) {
            Some(value) => value,
            None => return Err(Exception::throw_message(&ctx, "node is not an element")),
        };

        let selector = match Selector::parse(&selector) {
            Ok(value) => value,
            Err(err) => return Err(Exception::throw_message(&ctx, &err.to_string())),
        };

        let mut elements = vec![];
        for child in element.select(&selector) {
            let element = NativeElement { id: child.id() };
            let cls = Class::instance(ctx.clone(), element)?;
            elements.push(cls);
        }

        Ok(elements)
    }
}

impl<'js> Trace<'js> for NativeElement {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

fn create_element<'js>(ctx: Ctx<'js>, name: String) -> rquickjs::Result<Class<'js, NativeElement>> {
    let user_data = get_user_data(&ctx)?;

    let qual_name = QualName::new(None, ns!(html), LocalName::from(name));
    let id = user_data
        .html
        .borrow_mut()
        .tree
        .orphan(scraper::Node::Element(Element::new(qual_name, Vec::new())))
        .id();
    let element = NativeElement { id };

    let cls = Class::instance(ctx.clone(), element)?;

    Ok(cls)
}

fn get_user_agent<'js>(ctx: Ctx<'js>) -> rquickjs::Result<String> {
    let user_data = get_user_data(&ctx)?;

    Ok(user_data.user_agent.clone())
}

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
struct ChallengeResponse {
    client_hashes: Vec<String>,

    #[serde(flatten)]
    other: HashMap<String, serde_json::Value>,
}

pub(super) fn solve(user_agent: &str, challenge: &str) -> Result<String, SolveError> {
    let challenge = BASE64_STANDARD.decode(challenge)?;
    let challenge = std::str::from_utf8(&challenge)?;

    let mut challenge_response = solve_script(user_agent, challenge)?;

    for hash in challenge_response.client_hashes.iter_mut() {
        let mut hasher = Sha256::new();
        hasher.update(&*hash);
        let hash_value = hasher.finalize();

        *hash = BASE64_STANDARD.encode(hash_value);
    }
    let value = serde_json::to_string(&challenge_response)?;
    let value = BASE64_STANDARD.encode(value);

    Ok(value)
}

fn solve_script(user_agent: &str, challenge: &str) -> Result<ChallengeResponse, SolveError> {
    let runtime = Runtime::new()?;
    let ctx = Context::full(&runtime)?;

    let mut script = String::new();
    writeln!(&mut script, "await (async function() {{").unwrap();
    writeln!(
        &mut script,
        "{host}",
        host = include_str!("./host_script.js")
    )
    .unwrap();
    writeln!(&mut script, "let value = await {challenge}").unwrap();
    writeln!(&mut script, "return JSON.stringify(value);").unwrap();
    writeln!(&mut script, "}})()").unwrap();

    let value = ctx.with(|ctx| {
        ctx.store_userdata(UserData {
            html: Html::new_document().into(),
            user_agent: user_agent.into(),
        })?;

        let globals = ctx.globals();
        globals.set("nativeCreateElement", Func::from(create_element))?;
        globals.set("nativeGetUserAgent", Func::from(get_user_agent))?;

        let promise: Promise = ctx.eval_promise(script).catch(&ctx)?;
        promise.finish::<()>().catch(&ctx)?;

        let result: Object = promise.result().unwrap().catch(&ctx)?;
        let value: String = result.get("value")?;
        Result::<String, SolveError>::Ok(value)
    })?;

    let response: ChallengeResponse = serde_json::from_str(&value)?;

    Ok(response)
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    struct TestInput {
        user_agent: String,
    }

    #[test]
    fn test() {
        let mut dir_iter = std::fs::read_dir("test_data/challenge").expect("failed to read dir");

        while let Some(entry) = dir_iter.next() {
            let entry = entry.expect("failed to get dir entry");
            let file_type = entry.file_type().expect("failed to get file type");
            if !file_type.is_dir() {
                continue;
            }

            let challenge_path = entry.path().join("raw.js");
            let challenge =
                std::fs::read_to_string(challenge_path).expect("failed to read challenge");

            let input_path = entry.path().join("input.json");
            let input = std::fs::read_to_string(input_path).expect("failed to read input");
            let input: TestInput =
                serde_json::from_str(&input).expect("failed to parse input json");

            let solution_path = entry.path().join("solution.json");
            let solution = std::fs::read_to_string(solution_path).expect("failed to read solution");
            let solution: ChallengeResponse =
                serde_json::from_str(&solution).expect("failed to parse solution");

            let test_solution = solve_script(input.user_agent.as_str(), &challenge)
                .expect("failed to solve challenge");

            // dbg!(&test_solution);
            // dbg!(&solution);

            assert!(test_solution == solution);
        }
    }
}
