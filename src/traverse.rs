// use async_recursion::async_recursion;
// use mongodb::Database;
// use serde_json::Value;

// pub struct TraverseSuccess {}
// pub struct TraverseError {
//     msg: String,
// }

// #[async_recursion]
// pub async fn traverse(
//     db: &Database,
//     path: String,
//     entry: &Value,
// ) -> Result<TraverseSuccess, TraverseError> {
//     // Short circuit if value is null
//     if entry.is_null() {
//         return Ok(TraverseSuccess {});
//     }

//     let validated_path = validate_path(&path);
//     let data_coll = db.collection("data");
//     let mut doc = bson::doc! {
//     "path": &validated_path,
//     };

//     // Handle all possible value types
//     if entry.is_boolean() {
//         let val = match entry.as_bool() {
//             Some(val) => val,
//             None => {
//                 return Err(TraverseError {
//                     msg: "could not extract bool from entry".to_string(),
//                 });
//             }
//         };
//         doc.insert("value", val);
//         doc.insert("data_type", "boolean");
//         match data_coll
//             .replace_one(
//                 bson::doc! {"path": &validated_path},
//                 doc,
//                 mongodb::options::ReplaceOptions::builder()
//                     .upsert(true)
//                     .build(),
//             )
//             .await
//         {
//             Ok(_) => Ok(TraverseSuccess {}),
//             Err(e) => Err(TraverseError {
//                 msg: format!("unable to insert the mongodb doc: {}", e),
//             }),
//         }
//     } else if entry.is_u64() {
//         let val = match entry.as_u64() {
//             Some(val) => val,
//             None => {
//                 return Err(TraverseError {
//                     msg: "could not extract u64 from entry".to_string(),
//                 });
//             }
//         };
//         doc.insert("value", val);
//         doc.insert("data_type", "u64");
//         match data_coll
//             .replace_one(
//                 bson::doc! {"path": &validated_path},
//                 doc,
//                 mongodb::options::ReplaceOptions::builder()
//                     .upsert(true)
//                     .build(),
//             )
//             .await
//         {
//             Ok(_) => Ok(TraverseSuccess {}),
//             Err(e) => Err(TraverseError {
//                 msg: format!("unable to insert the mongodb doc: {}", e),
//             }),
//         }
//     } else if entry.is_i64() {
//         let val = match entry.as_i64() {
//             Some(val) => val,
//             None => {
//                 return Err(TraverseError {
//                     msg: "could not extract i64 from entry".to_string(),
//                 });
//             }
//         };
//         doc.insert("value", val);
//         doc.insert("data_type", "i64");
//         match data_coll
//             .replace_one(
//                 bson::doc! {"path": &validated_path},
//                 doc,
//                 mongodb::options::ReplaceOptions::builder()
//                     .upsert(true)
//                     .build(),
//             )
//             .await
//         {
//             Ok(_) => Ok(TraverseSuccess {}),
//             Err(e) => Err(TraverseError {
//                 msg: format!("unable to insert the mongodb doc: {}", e),
//             }),
//         }
//     } else if entry.is_f64() {
//         let val = match entry.as_f64() {
//             Some(val) => val,
//             None => {
//                 return Err(TraverseError {
//                     msg: "could not extract f64 from entry".to_string(),
//                 });
//             }
//         };
//         doc.insert("value", val);
//         doc.insert("data_type", "f64");
//         match data_coll
//             .replace_one(
//                 bson::doc! {"path": &validated_path},
//                 doc,
//                 mongodb::options::ReplaceOptions::builder()
//                     .upsert(true)
//                     .build(),
//             )
//             .await
//         {
//             Ok(_) => Ok(TraverseSuccess {}),
//             Err(e) => Err(TraverseError {
//                 msg: format!("unable to insert the mongodb doc: {}", e),
//             }),
//         }
//     } else if entry.is_string() {
//         let val = match entry.as_str() {
//             Some(val) => val,
//             None => {
//                 return Err(TraverseError {
//                     msg: "could not extract string from entry".to_string(),
//                 });
//             }
//         };
//         doc.insert("value", val);
//         doc.insert("data_type", "string");
//         match data_coll
//             .replace_one(
//                 bson::doc! {"path": &validated_path},
//                 doc,
//                 mongodb::options::ReplaceOptions::builder()
//                     .upsert(true)
//                     .build(),
//             )
//             .await
//         {
//             Ok(_) => Ok(TraverseSuccess {}),
//             Err(e) => Err(TraverseError {
//                 msg: format!("unable to insert the mongodb doc: {}", e),
//             }),
//         }
//     } else if entry.is_array() {
//         let arr = match entry.as_array() {
//             Some(arr) => arr,
//             None => {
//                 return Err(TraverseError {
//                     msg: "could not be interpreted as array".to_string(),
//                 });
//             }
//         };
//         match futures::future::try_join_all(
//             arr.iter()
//                 .map(|elem| traverse(db, validated_path.clone(), elem)),
//         )
//         .await
//         {
//             Ok(_) => Ok(TraverseSuccess {}),
//             Err(e) => Err(e),
//         }
//     } else if entry.is_object() {
//         let obj = match entry.as_object() {
//             Some(obj) => obj,
//             None => {
//                 return Err(TraverseError {
//                     msg: "could not be interpreted as object".to_string(),
//                 });
//             }
//         };

//         match futures::future::try_join_all(
//             obj.iter()
//                 .map(|(key, val)| traverse(db, format!("{}{}.", &validated_path, key), val)),
//         )
//         .await
//         {
//             Ok(_) => Ok(TraverseSuccess {}),
//             Err(e) => Err(e),
//         }
//     } else {
//         Ok(TraverseSuccess {})
//     }
// }
