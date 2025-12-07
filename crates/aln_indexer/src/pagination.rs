pub struct Cursor { pub block_height: i64, pub account_id: i64, pub denom_id: i64 }

pub fn next_cursor(_last: &Cursor) -> Cursor { Cursor { block_height: _last.block_height, account_id: _last.account_id, denom_id: _last.denom_id } }

pub fn keyset_paging_query(block_height: i64, last_cursor: Option<Cursor>, limit: i64) -> (String, Vec<String>) {
    // For example, build SQL like: WHERE block_height = $1 and (account_id, denom_id) > ($2, $3)
    if let Some(cur) = last_cursor {
        let q = "SELECT * FROM balance_snapshot WHERE block_height = $1 AND (account_id, denom_id) > ($2, $3) ORDER BY account_id, denom_id LIMIT $4".to_string();
        return (q, vec![block_height.to_string(), cur.account_id.to_string(), cur.denom_id.to_string(), limit.to_string()]);
    }
    let q = "SELECT * FROM balance_snapshot WHERE block_height = $1 ORDER BY account_id, denom_id LIMIT $2".to_string();
    return (q, vec![block_height.to_string(), limit.to_string()]);
}
