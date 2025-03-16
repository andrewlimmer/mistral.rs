use std::sync::Arc;

use anyhow::Result;
use llguidance::{
    api::{ParserLimits, RegexNode, TopLevelGrammar},
    lark_to_llguidance,
    toktrie::{InferenceCapabilities, TokEnv},
    JsonCompileOptions, TokenParser,
};
use tokenizers::Tokenizer;

use crate::Constraint;

pub fn build_tok_env(tokenizer: Tokenizer, eos: Option<String>) -> TokEnv {
    let bt = toktrie_hf_tokenizers::ByteTokenizer::from_tokenizer(tokenizer)
        .expect("Failed to create ByteTokenizer from Tokenizer");
    let mut env = toktrie_hf_tokenizers::ByteTokenizerEnv::new(bt, None)
        .expect("Failed to create ByteTokenizerEnv");

    // Force eos token
    if let Some(eos) = eos {
        println!(
            "eos:{:?}, {:?}",
            eos.as_str(),
            env.tok_trie.get_special_token(eos.as_str()).unwrap()
        );
        env.tok_trie
            .with_eos_token(env.tok_trie.get_special_token(eos.as_str()).unwrap());
        env.tok_trie = env.tok_trie.build_chat_mode_trie();
    }

    Arc::new(env)
}

pub fn llg_grammar_from_constraint(constraint: &Constraint) -> Result<Option<TopLevelGrammar>> {
    let grm = match constraint {
        Constraint::Regex(regex) => {
            TopLevelGrammar::from_regex(RegexNode::Regex(regex.to_string()))
        }
        Constraint::Lark(lark) => lark_to_llguidance(lark)?,
        Constraint::JsonSchema(value) => {
            JsonCompileOptions::default().json_to_llg_no_validate(value.clone())?
        }
        Constraint::Llguidance(value) => value.clone(),
        Constraint::None => return Ok(None),
    };
    Ok(Some(grm))
}

pub fn constraint_from_llg_grammar(
    tok_env: TokEnv,
    grm: TopLevelGrammar,
) -> Result<llguidance::Constraint> {
    let parser = TokenParser::from_llguidance_json(
        tok_env,
        grm,
        llguidance::Logger::new(0, 1),
        InferenceCapabilities {
            ..Default::default()
        },
        ParserLimits::default(),
        vec![],
    )?;
    Ok(llguidance::Constraint::new(parser))
}
