use crate::transaction::models::HeaderPairRecord;
use crate::transaction::models::TransactionIdempotent;
use actix_web::{HttpResponse, body::to_bytes};
use error_stack::Report;

#[derive(Debug, thiserror::Error)]
#[error("Bad Body: {0}")]
pub struct BytesConvertError(pub String);

pub async fn response_to_tx_idempotent(
    account_id: uuid::Uuid,
    amount: f64,
    transaction_ref: &str,
    http_res: HttpResponse,
) -> Result<(HttpResponse, TransactionIdempotent), Report<BytesConvertError>> {
    let (response_head, body) = http_res.into_parts();
    let body = to_bytes(body)
        .await
        .map_err(|e| BytesConvertError(e.to_string()))?;

    let status_code = response_head.status().as_u16() as i16;

    let headers = {
        let mut h = Vec::with_capacity(response_head.headers().len());
        for (name, value) in response_head.headers().iter() {
            let name = name.as_str().to_owned();
            let value = value.as_bytes().to_owned();
            h.push(HeaderPairRecord { name, value })
        }
        h
    };

    let tx_idem = TransactionIdempotent::new(
        account_id,
        transaction_ref.to_owned(),
        amount.to_owned(),
        status_code,
        headers,
        body.as_ref().to_owned(),
    );

    let http_res = response_head.set_body(body).map_into_boxed_body();

    Ok((http_res, tx_idem))
}
