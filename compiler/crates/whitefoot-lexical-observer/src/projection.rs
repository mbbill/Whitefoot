use whitefoot_contract::{SourceBundle, SourceId};
use whitefoot_lexer::{
    LexCompilerFailure, LexLimit, LexOutcome, LexResourceFailure, LexStorage, Lexeme,
    SourceIssueKind, TokenKind, TriviaKind,
};

use crate::ACTIVE_KERNEL_SPEC_HASH;
use crate::protocol::{AdapterError, RESPONSE_MAGIC, RESPONSE_VERSION, ResponseEncoder};

pub(crate) fn encode_observation(
    source: &SourceBundle,
    outcome: LexOutcome<'_>,
) -> Result<Vec<u8>, AdapterError> {
    let response_length = encoded_length(source, &outcome)?;
    let mut encoder = ResponseEncoder::with_capacity(response_length)?
        .bytes(&RESPONSE_MAGIC)?
        .u16(RESPONSE_VERSION)?
        .bytes(ACTIVE_KERNEL_SPEC_HASH.digest().as_bytes())?;
    encoder = match outcome {
        LexOutcome::Complete(lexed) => {
            let source_count =
                u32::try_from(source.len()).map_err(|_| AdapterError::ProjectionInvariant)?;
            let mut encoded = encoder.u8(0)?.u64(lexed.token_count())?.u32(source_count)?;
            for source_ordinal in 0..source_count {
                let source_id = SourceId::from_ordinal(source_ordinal);
                let lexemes = lexed
                    .source_lexemes(source_id)
                    .ok_or(AdapterError::ProjectionInvariant)?;
                let count =
                    u64::try_from(lexemes.len()).map_err(|_| AdapterError::ProjectionInvariant)?;
                encoded = encoded.u64(count)?;
                for lexeme in lexemes {
                    let span = lexeme.span();
                    if span.source() != source_id {
                        return Err(AdapterError::ProjectionInvariant);
                    }
                    encoded = encoded
                        .u8(lexeme_tag(*lexeme))?
                        .u64(span.start().value())?
                        .u64(span.end().value())?;
                }
            }
            encoded
        }
        LexOutcome::SourceIssue(issue) => encoder
            .u8(1)?
            .u32(issue.span().source().ordinal())?
            .u64(issue.span().start().value())?
            .u64(issue.span().end().value())?
            .u8(source_issue_tag(issue.kind()))?,
        LexOutcome::ResourceFailure(failure) => encode_resource_failure(encoder.u8(2)?, failure)?,
        LexOutcome::CompilerFailure(failure) => encode_compiler_failure(encoder.u8(3)?, failure)?,
    };
    encoder.finish()
}

fn encoded_length(source: &SourceBundle, outcome: &LexOutcome<'_>) -> Result<u64, AdapterError> {
    const HEADER: u64 = 8 + 2 + 32;
    let payload = match outcome {
        LexOutcome::Complete(lexed) => {
            let sources =
                u64::try_from(source.len()).map_err(|_| AdapterError::ResponseLimitExceeded)?;
            let lexemes = u64::try_from(lexed.lexemes().len())
                .map_err(|_| AdapterError::ResponseLimitExceeded)?;
            1_u64
                .checked_add(8 + 4)
                .and_then(|length| length.checked_add(sources.checked_mul(8)?))
                .and_then(|length| length.checked_add(lexemes.checked_mul(17)?))
                .ok_or(AdapterError::ResponseLimitExceeded)?
        }
        LexOutcome::SourceIssue(_) => 1 + 4 + 8 + 8 + 1,
        LexOutcome::ResourceFailure(failure) => match failure {
            LexResourceFailure::LimitExceeded { .. } => 1 + 1 + 1 + 8 + 8,
            LexResourceFailure::AddressSpaceExceeded { .. }
            | LexResourceFailure::StorageUnavailable { .. } => 1 + 1 + 1 + 8,
        },
        LexOutcome::CompilerFailure(failure) => match failure {
            LexCompilerFailure::InvalidProducedSpan { .. } => 1 + 1 + 4 + 8 + 8,
            LexCompilerFailure::PassDisagreement { .. } => 1 + 1 + 4,
            LexCompilerFailure::PassCountDisagreement { .. } => 1 + 1 + 8 + 8 + 8 + 8,
            LexCompilerFailure::CounterOverflow => 1 + 1,
        },
    };
    HEADER
        .checked_add(payload)
        .ok_or(AdapterError::ResponseLimitExceeded)
}

const fn lexeme_tag(lexeme: Lexeme<'_>) -> u8 {
    match lexeme {
        Lexeme::Token(token) => token_kind_tag(token.kind()),
        Lexeme::Trivia(trivia) => match trivia.kind() {
            TriviaKind::Spaces => 23,
            TriviaKind::LineFeed => 24,
        },
    }
}

const fn token_kind_tag(kind: TokenKind) -> u8 {
    match kind {
        TokenKind::LowerWordForm => 0,
        TokenKind::UpperWordForm => 1,
        TokenKind::RegionForm => 2,
        TokenKind::LabelForm => 3,
        TokenKind::OperationNameForm => 4,
        TokenKind::NumberForm => 5,
        TokenKind::StringForm => 6,
        TokenKind::LeftParen => 7,
        TokenKind::RightParen => 8,
        TokenKind::LeftBrace => 9,
        TokenKind::RightBrace => 10,
        TokenKind::LeftBracket => 11,
        TokenKind::RightBracket => 12,
        TokenKind::LeftAngle => 13,
        TokenKind::RightAngle => 14,
        TokenKind::Comma => 15,
        TokenKind::Colon => 16,
        TokenKind::Semicolon => 17,
        TokenKind::Dot => 18,
        TokenKind::Equal => 19,
        TokenKind::ThinArrow => 20,
        TokenKind::FatArrow => 21,
        TokenKind::Ampersand => 22,
    }
}

const fn source_issue_tag(kind: SourceIssueKind) -> u8 {
    match kind {
        SourceIssueKind::InvalidUtf8 => 0,
        SourceIssueKind::UnexpectedByte => 1,
        SourceIssueKind::MissingRegionName => 2,
        SourceIssueKind::MissingLabelName => 3,
        SourceIssueKind::UnterminatedString => 4,
        SourceIssueKind::InvalidStringByte => 5,
        SourceIssueKind::InvalidStringEscape => 6,
    }
}

const fn limit_tag(limit: LexLimit) -> u8 {
    match limit {
        LexLimit::Sources => 0,
        LexLimit::SourceBytes => 1,
        LexLimit::TotalSourceBytes => 2,
        LexLimit::TokenBytes => 3,
        LexLimit::Tokens => 4,
        LexLimit::Lexemes => 5,
    }
}

const fn storage_tag(storage: LexStorage) -> u8 {
    match storage {
        LexStorage::Lexemes => 0,
        LexStorage::SourceBoundaries => 1,
    }
}

fn encode_resource_failure(
    encoder: ResponseEncoder,
    failure: LexResourceFailure,
) -> Result<ResponseEncoder, AdapterError> {
    match failure {
        LexResourceFailure::LimitExceeded {
            limit,
            maximum,
            actual,
        } => encoder
            .u8(0)?
            .u8(limit_tag(limit))?
            .u64(maximum)?
            .u64(actual),
        LexResourceFailure::AddressSpaceExceeded { storage, requested } => {
            encoder.u8(1)?.u8(storage_tag(storage))?.u64(requested)
        }
        LexResourceFailure::StorageUnavailable { storage, requested } => {
            encoder.u8(2)?.u8(storage_tag(storage))?.u64(requested)
        }
    }
}

fn encode_compiler_failure(
    encoder: ResponseEncoder,
    failure: LexCompilerFailure,
) -> Result<ResponseEncoder, AdapterError> {
    match failure {
        LexCompilerFailure::InvalidProducedSpan { source, start, end } => encoder
            .u8(0)?
            .u32(source.ordinal())?
            .u64(start.value())?
            .u64(end.value()),
        LexCompilerFailure::PassDisagreement { source } => encoder.u8(1)?.u32(source.ordinal()),
        LexCompilerFailure::PassCountDisagreement {
            expected_lexemes,
            actual_lexemes,
            expected_tokens,
            actual_tokens,
        } => encoder
            .u8(2)?
            .u64(expected_lexemes)?
            .u64(actual_lexemes)?
            .u64(expected_tokens)?
            .u64(actual_tokens),
        LexCompilerFailure::CounterOverflow => encoder.u8(3),
    }
}
