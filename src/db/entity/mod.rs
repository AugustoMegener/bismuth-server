
pub mod facets;


pub const UUID_DEFAULT: &'static str = concat!(
    r#"(lower(hex(randomblob(4))) || '-' || "#,
		r#"lower(hex(randomblob(2))) || '-4' || "#,
		r#"substr(lower(hex(randomblob(2))),2) || '-' || "#,
		r#"substr('89ab',abs(random()) % 4 + 1,1) || substr(lower(hex(randomblob(2))),2) || '-' || "#,
		r#"lower(hex(randomblob(6))))"#,
);
