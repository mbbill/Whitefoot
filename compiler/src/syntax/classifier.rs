use crate::SpecHash;
use crate::lexer::{LexedBundle, Lexeme, Token, TokenKind};
use crate::syntax::terminal::{
    FixedTerminalV0_14, TERMINAL_CONTRACT_SPEC_V0_14, TerminalPredicateV0_14, TerminalSetV0_14,
    is_digits_v0_14, is_identifier_v0_14, is_label_v0_14, is_literal_v0_14,
    is_operation_name_v0_14, is_region_identifier_v0_14, is_string_v0_14, is_type_identifier_v0_14,
};

use crate::syntax::outcome::{
    ClassifiedBundle, ClassifiedToken, TerminalCompilerFailure, TerminalInvocationFailure,
    TerminalIssue, TerminalIssueOwner, TerminalLimit, TerminalLimits, TerminalOutcome,
    TerminalResourceFailure, TerminalStorage,
};

fn reserve_exact<T>(
    count: u64,
    storage: TerminalStorage,
) -> Result<Vec<T>, TerminalResourceFailure> {
    let count_usize =
        usize::try_from(count).map_err(|_| TerminalResourceFailure::AddressSpaceExceeded {
            storage,
            requested: count,
        })?;
    let mut output = Vec::new();
    output.try_reserve_exact(count_usize).map_err(|_| {
        TerminalResourceFailure::StorageUnavailable {
            storage,
            requested: count,
        }
    })?;
    Ok(output)
}

fn fixed(set: &mut TerminalSetV0_14, terminal: FixedTerminalV0_14, spelling: &[u8]) -> bool {
    if spelling != terminal.spelling() {
        return false;
    }
    set.insert(TerminalPredicateV0_14::Fixed(terminal));
    true
}

fn membership(token: Token<'_>) -> Option<TerminalSetV0_14> {
    let spelling = token.span().bytes();
    let mut set = TerminalSetV0_14::empty();
    let valid_shape = match token.kind() {
        TokenKind::LowerWordForm => {
            if let Some(terminal) = FixedTerminalV0_14::from_spelling(spelling) {
                set.insert(TerminalPredicateV0_14::Fixed(terminal));
                if is_literal_v0_14(spelling) {
                    set.insert(TerminalPredicateV0_14::Literal);
                }
                true
            } else if is_identifier_v0_14(spelling) {
                set.insert(TerminalPredicateV0_14::Identifier);
                true
            } else {
                false
            }
        }
        TokenKind::UpperWordForm => {
            let valid = is_type_identifier_v0_14(spelling);
            if valid {
                set.insert(TerminalPredicateV0_14::TypeIdentifier);
            }
            valid
        }
        TokenKind::RegionForm => {
            let valid = is_region_identifier_v0_14(spelling);
            if valid {
                set.insert(TerminalPredicateV0_14::RegionIdentifier);
            }
            valid
        }
        TokenKind::LabelForm => {
            let valid = is_label_v0_14(spelling);
            if valid {
                set.insert(TerminalPredicateV0_14::Label);
            }
            valid
        }
        TokenKind::OperationNameForm => {
            let valid = is_operation_name_v0_14(spelling);
            if valid {
                set.insert(TerminalPredicateV0_14::OperationName);
            }
            valid
        }
        TokenKind::NumberForm => {
            if is_literal_v0_14(spelling) {
                set.insert(TerminalPredicateV0_14::Literal);
            }
            if is_digits_v0_14(spelling) {
                set.insert(TerminalPredicateV0_14::Digits);
            }
            true
        }
        TokenKind::StringForm => {
            let valid = is_string_v0_14(spelling);
            if valid {
                set.insert(TerminalPredicateV0_14::String);
            }
            valid
        }
        TokenKind::LeftParen => fixed(&mut set, FixedTerminalV0_14::LeftParen, spelling),
        TokenKind::RightParen => fixed(&mut set, FixedTerminalV0_14::RightParen, spelling),
        TokenKind::LeftBrace => fixed(&mut set, FixedTerminalV0_14::LeftBrace, spelling),
        TokenKind::RightBrace => fixed(&mut set, FixedTerminalV0_14::RightBrace, spelling),
        TokenKind::LeftBracket => fixed(&mut set, FixedTerminalV0_14::LeftBracket, spelling),
        TokenKind::RightBracket => fixed(&mut set, FixedTerminalV0_14::RightBracket, spelling),
        TokenKind::LeftAngle => fixed(&mut set, FixedTerminalV0_14::LeftAngle, spelling),
        TokenKind::RightAngle => fixed(&mut set, FixedTerminalV0_14::RightAngle, spelling),
        TokenKind::Comma => fixed(&mut set, FixedTerminalV0_14::Comma, spelling),
        TokenKind::Colon => fixed(&mut set, FixedTerminalV0_14::Colon, spelling),
        TokenKind::Semicolon => fixed(&mut set, FixedTerminalV0_14::Semicolon, spelling),
        TokenKind::Dot => fixed(&mut set, FixedTerminalV0_14::Dot, spelling),
        TokenKind::Equal => fixed(&mut set, FixedTerminalV0_14::Equal, spelling),
        TokenKind::ThinArrow => fixed(&mut set, FixedTerminalV0_14::ThinArrow, spelling),
        TokenKind::FatArrow => fixed(&mut set, FixedTerminalV0_14::FatArrow, spelling),
        TokenKind::Ampersand => fixed(&mut set, FixedTerminalV0_14::Ampersand, spelling),
    };
    (valid_shape && !set.is_empty()).then_some(set)
}

fn invalid_token(token: Token<'_>) -> TerminalCompilerFailure {
    TerminalCompilerFailure::InvalidFormedToken {
        source: token.id().source(),
        start: token.id().start(),
        end: token.id().end(),
    }
}

/// Applies every exact v0.14 terminal predicate to every formed token.
///
/// Classification is context-free and failure-atomic. It never consults a
/// parser position, another token, name lookup, or the operation table, and it
/// retains all matching predicates rather than choosing one by priority.
#[must_use]
pub fn classify_terminals_v0_14<'lexed, 'source>(
    lexed: &'lexed LexedBundle<'source>,
    specification: SpecHash,
    limits: TerminalLimits,
) -> TerminalOutcome<'lexed, 'source> {
    if specification != TERMINAL_CONTRACT_SPEC_V0_14 {
        return TerminalOutcome::InvocationFailure(
            TerminalInvocationFailure::SpecificationMismatch {
                expected: TERMINAL_CONTRACT_SPEC_V0_14,
                actual: specification,
            },
        );
    }

    let token_count = lexed.token_count();
    if token_count > limits.max_tokens {
        return TerminalOutcome::ResourceFailure(TerminalResourceFailure::LimitExceeded {
            limit: TerminalLimit::Tokens,
            maximum: limits.max_tokens,
            actual: token_count,
        });
    }

    let boundary_count = match u64::try_from(lexed.source_bundle().len())
        .ok()
        .and_then(|count| count.checked_add(1))
    {
        Some(count) => count,
        None => {
            return TerminalOutcome::CompilerFailure(TerminalCompilerFailure::CounterOverflow);
        }
    };
    let mut tokens = match reserve_exact(token_count, TerminalStorage::Tokens) {
        Ok(tokens) => tokens,
        Err(failure) => return TerminalOutcome::ResourceFailure(failure),
    };
    let mut source_offsets = match reserve_exact(boundary_count, TerminalStorage::SourceBoundaries)
    {
        Ok(offsets) => offsets,
        Err(failure) => return TerminalOutcome::ResourceFailure(failure),
    };
    source_offsets.push(0);

    let mut actual_count = 0_u64;
    for (source, _) in lexed.source_bundle().iter() {
        let Some(lexemes) = lexed.source_lexemes(source) else {
            return TerminalOutcome::CompilerFailure(
                TerminalCompilerFailure::MissingSourcePartition { source },
            );
        };
        for lexeme in lexemes {
            let Lexeme::Token(token) = lexeme else {
                continue;
            };
            let Some(terminals) = membership(*token) else {
                if token.kind() == TokenKind::NumberForm {
                    return TerminalOutcome::SourceIssue(TerminalIssue {
                        token: token.id(),
                        owner: TerminalIssueOwner::Form5,
                    });
                }
                return TerminalOutcome::CompilerFailure(invalid_token(*token));
            };
            actual_count = match actual_count.checked_add(1) {
                Some(count) => count,
                None => {
                    return TerminalOutcome::CompilerFailure(
                        TerminalCompilerFailure::CounterOverflow,
                    );
                }
            };
            tokens.push(ClassifiedToken {
                token: *token,
                terminals,
            });
        }
        source_offsets.push(tokens.len());
    }

    if actual_count != token_count {
        return TerminalOutcome::CompilerFailure(TerminalCompilerFailure::TokenCountDisagreement {
            expected: token_count,
            actual: actual_count,
        });
    }

    TerminalOutcome::Complete(ClassifiedBundle {
        spec: specification,
        lexed,
        tokens,
        source_offsets,
    })
}
