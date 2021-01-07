use crate::filters::Data;
use mongodb::Database;
use serde_json::Value;

pub struct TraverseSuccess {}
pub struct TraverseError {
    msg: String,
}

pub fn insert<'a>(
    db: &'a Database,
    data: &'a Data,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<TraverseSuccess, TraverseError>> + 'a + Send>,
> {
    let data_coll = db.collection("data");
    let validated_path = validate_path(&data.path);

    let mut doc = bson::doc! {
    "path": &validated_path,
    "data_type": &data.data_type,
    };

    // Handle all possible value types
    if data.value.is_boolean() {
        let val = match data.value.as_bool() {
            Some(val) => val,
            None => {
                return Box::pin(async move {
                    Err(TraverseError {
                        msg: "could not extract bool from data.value".to_string(),
                    })
                });
            }
        };
        doc.insert("value", val);
    } else if data.value.is_u64() {
        let val = match data.value.as_u64() {
            Some(val) => val,
            None => {
                return Box::pin(async move {
                    Err(TraverseError {
                        msg: "could not extract u64 from data.value".to_string(),
                    })
                });
            }
        };
        doc.insert("value", val);
    } else if data.value.is_i64() {
        let val = match data.value.as_i64() {
            Some(val) => val,
            None => {
                return Box::pin(async move {
                    Err(TraverseError {
                        msg: "could not extract i64 from data.value".to_string(),
                    })
                });
            }
        };
        doc.insert("value", val);
    } else if data.value.is_f64() {
        let val = match data.value.as_f64() {
            Some(val) => val,
            None => {
                return Box::pin(async move {
                    Err(TraverseError {
                        msg: "could not extract f64 from data.value".to_string(),
                    })
                });
            }
        };
        doc.insert("value", val);
    } else if data.value.is_string() {
        let val = match data.value.as_str() {
            Some(val) => val,
            None => {
                return Box::pin(async move {
                    Err(TraverseError {
                        msg: "could not extract string from data.value".to_string(),
                    })
                });
            }
        };
        doc.insert("value", val);
    } else {
        return Box::pin(async move {
            Err(TraverseError {
                msg: "data is not of a valid type".to_string(),
            })
        });
    }

    Box::pin(async move {
        match data_coll
            .replace_one(
                bson::doc! {"path": &validated_path},
                doc,
                mongodb::options::ReplaceOptions::builder()
                    .upsert(true)
                    .build(),
            )
            .await
        {
            Ok(_) => Ok(TraverseSuccess {}),
            Err(e) => Err(TraverseError {
                msg: format!("unable to insert the mongodb doc: {}", e),
            }),
        }
    })
}

pub fn traverse<'a>(
    db: &'a Database,
    path: &str,
    entry: &'a Value,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<TraverseSuccess, TraverseError>> + 'a + Send>,
> {
    // Short circuit if value is null
    if entry.is_null() {
        return Box::pin(async move { Ok(TraverseSuccess {}) });
    }

    let validated_path = validate_path(path);
    let data_coll = db.collection("data");
    let mut doc = bson::doc! {
    "path": &validated_path,
    };

    // Handle all possible value types
    if entry.is_boolean() {
        let val = match entry.as_bool() {
            Some(val) => val,
            None => {
                return Box::pin(async move {
                    Err(TraverseError {
                        msg: "could not extract bool from entry".to_string(),
                    })
                });
            }
        };
        doc.insert("value", val);
        doc.insert("data_type", "boolean");
        Box::pin(async move {
            match data_coll
                .replace_one(
                    bson::doc! {"path": &validated_path},
                    doc,
                    mongodb::options::ReplaceOptions::builder()
                        .upsert(true)
                        .build(),
                )
                .await
            {
                Ok(_) => Ok(TraverseSuccess {}),
                Err(e) => Err(TraverseError {
                    msg: format!("unable to insert the mongodb doc: {}", e),
                }),
            }
        })
    } else if entry.is_u64() {
        let val = match entry.as_u64() {
            Some(val) => val,
            None => {
                return Box::pin(async move {
                    Err(TraverseError {
                        msg: "could not extract u64 from entry".to_string(),
                    })
                });
            }
        };
        doc.insert("value", val);
        doc.insert("data_type", "u64");
        Box::pin(async move {
            match data_coll
                .replace_one(
                    bson::doc! {"path": &validated_path},
                    doc,
                    mongodb::options::ReplaceOptions::builder()
                        .upsert(true)
                        .build(),
                )
                .await
            {
                Ok(_) => Ok(TraverseSuccess {}),
                Err(e) => Err(TraverseError {
                    msg: format!("unable to insert the mongodb doc: {}", e),
                }),
            }
        })
    } else if entry.is_i64() {
        let val = match entry.as_i64() {
            Some(val) => val,
            None => {
                return Box::pin(async move {
                    Err(TraverseError {
                        msg: "could not extract i64 from entry".to_string(),
                    })
                });
            }
        };
        doc.insert("value", val);
        doc.insert("data_type", "i64");
        Box::pin(async move {
            match data_coll
                .replace_one(
                    bson::doc! {"path": &validated_path},
                    doc,
                    mongodb::options::ReplaceOptions::builder()
                        .upsert(true)
                        .build(),
                )
                .await
            {
                Ok(_) => Ok(TraverseSuccess {}),
                Err(e) => Err(TraverseError {
                    msg: format!("unable to insert the mongodb doc: {}", e),
                }),
            }
        })
    } else if entry.is_f64() {
        let val = match entry.as_f64() {
            Some(val) => val,
            None => {
                return Box::pin(async move {
                    Err(TraverseError {
                        msg: "could not extract f64 from entry".to_string(),
                    })
                });
            }
        };
        doc.insert("value", val);
        doc.insert("data_type", "f64");
        Box::pin(async move {
            match data_coll
                .replace_one(
                    bson::doc! {"path": &validated_path},
                    doc,
                    mongodb::options::ReplaceOptions::builder()
                        .upsert(true)
                        .build(),
                )
                .await
            {
                Ok(_) => Ok(TraverseSuccess {}),
                Err(e) => Err(TraverseError {
                    msg: format!("unable to insert the mongodb doc: {}", e),
                }),
            }
        })
    } else if entry.is_string() {
        let val = match entry.as_str() {
            Some(val) => val,
            None => {
                return Box::pin(async move {
                    Err(TraverseError {
                        msg: "could not extract string from entry".to_string(),
                    })
                });
            }
        };
        doc.insert("value", val);
        doc.insert("data_type", "string");
        Box::pin(async move {
            match data_coll
                .replace_one(
                    bson::doc! {"path": &validated_path},
                    doc,
                    mongodb::options::ReplaceOptions::builder()
                        .upsert(true)
                        .build(),
                )
                .await
            {
                Ok(_) => Ok(TraverseSuccess {}),
                Err(e) => Err(TraverseError {
                    msg: format!("unable to insert the mongodb doc: {}", e),
                }),
            }
        })
    } else if entry.is_array() {
        let arr = match entry.as_array() {
            Some(arr) => arr,
            None => {
                return Box::pin(async move {
                    Err(TraverseError {
                        msg: "could not be interpreted as array".to_string(),
                    })
                })
            }
        };
        Box::pin(async move {
            match futures::future::try_join_all(
                arr.iter().map(|elem| traverse(db, &validated_path, elem)),
            )
            .await
            {
                Ok(_) => Ok(TraverseSuccess {}),
                Err(e) => Err(e),
            }
        })
    } else if entry.is_object() {
        let obj = match entry.as_object() {
            Some(obj) => obj,
            None => {
                return Box::pin(async move {
                    Err(TraverseError {
                        msg: "could not be interpreted as object".to_string(),
                    })
                });
            }
        };

        Box::pin(async move {
            match futures::future::try_join_all(
                obj.iter()
                    .map(|(key, val)| traverse(db, &format!("{}{}.", &validated_path, key), val)),
            )
            .await
            {
                Ok(_) => Ok(TraverseSuccess {}),
                Err(e) => Err(e),
            }
        })
    } else {
        Box::pin(async move { Ok(TraverseSuccess {}) })
    }
}

// Ensures that a data entry path begins and ends with a period ('.')
// Empty strings will return as "."
// Strings of length 1 where the only char is a period will return as "."
// All other strings will have periods added to the beginning or end if needed.
// For now, string containing multiple periods in a row, or composed only of
// multiple periods, will be accepted and returned as given, with the same
// behavior as any other standard string of len > 1.
// This function is implemented as a boolean circuit to avoid iterating through
// the same string numerous times.
pub fn validate_path(path: &str) -> String {
    // Short circuit if path is empty
    if path.len() == 0 {
        return ".".to_string();
    }

    // Collect the first and last characters of the path
    let mut path_chars = path.chars();
    let first_char = path_chars.nth(0);
    let last_char = path_chars.last();

    // Match on the results of char extraction
    match (first_char, last_char) {
        // String length >= 2
        (Some(fc), Some(lc)) => {
            if fc != '.' && lc != '.' {
                format!(".{}.", path)
            } else if fc == '.' && lc == '.' {
                path.to_string()
            } else if fc != '.' {
                format!(".{}", path)
            } else {
                format!("{}.", path)
            }
        }
        // String length == 1
        (Some(fc), None) => {
            if fc == '.' {
                path.to_string()
            } else {
                format!(".{}.", path)
            }
        }
        // Impossible case: string length == 0, should never be here because
        // of the short-circuit implemented at the beginning of the function
        (None, None) => panic!(
            "this is an impossible situation; if you have gotten here, \\
	     a short-circuit earlier in the function has failed to function as \\
	     intended"
        ),
        // Impossible case: if this happens we should panic because something is
        // fundamentally wrong with the computing environment and someone should
        // know about it.
        // If the last char is != None, then it MUST BE that the
        // first char is != None, as the last char is collected after the
        // iterator has ticked over one spot to account for the first char,
        // therefore if the iterator finds something in the last() call, then
        // it must be after having collected something from the nth(0) call.
        (None, Some(_)) => panic!(
            "this is an impossible situation; if you have gotten here, \\
	     something has happened that should never happen according to the \\
	     laws of computing and/or the rust compiler. if you have gotten here, \\
	     some major memory or computing trickery has occurred and you should \\
	     be concerned for the integrity of your computing base"
        ),
    }
}
