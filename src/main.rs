extern crate serde;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use serde_json::Error;
use serde_json::Map;
use serde_json::Value;
use std::fmt::{self};

use std::collections::HashMap;

use std::collections::hash_map::Entry::{Occupied, Vacant};


//#[derive(Serialize, Deserialize)]
struct JsonApiRoot {
    data: Option<Map<String, Value>>,
    included: Option<Map<String, Value>>,
    jsonapi: Option<Map<String, Value>>,
    links: Option<Map<String, Value>>,
    meta: Option<Map<String, Value>>,
    root_items: Option<Vec<JsonApiObject>>,
}

impl JsonApiRoot {
    pub fn new() -> JsonApiRoot {
        JsonApiRoot {
            data: None,
            included: None,
            jsonapi:None,
            links: None,
            meta: None,
            root_items: None,
        }
    }

    fn add_root_item(&mut self, json_api_object: JsonApiObject) {
        if let Some(ref mut rels) = self.root_items {
            rels.push(json_api_object);
        }
    }

    pub fn serialize(self) -> Result<String, Error> {
        let mut json_api_root = Map::new();
        //json_api_root.insert(String::from("data"), self.data);
        //json_api_root.insert(String::from("included"), self.data);
        json_api_root.insert(String::from("links"), self.links());
        json_api_root.insert(String::from("meta"), self.meta());
        json_api_root.insert(String::from("jsonapi"), self.jsonapi());
        let res = serde_json::to_string(&json_api_root);
        return res;
    }

    fn links(&self) -> Value {
        let map: Map<String, Value> = Map::new();
        return map.into();
    }

    fn meta(&self) -> Value {
        let map: Map<String, Value> = Map::new();
        return serde_json::to_value(map).unwrap();
    }

    fn jsonapi(&self) -> Value {
        return json!({ "version": "1.0" })
    }
}

//#[derive(Clone)]
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
enum JsonApiType {
    User,
    Event,
    Rsvp,
    Message,
    Login,
    Device,
    Friend,
    UserIdList,
}

impl fmt::Display for JsonApiType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            JsonApiType::User => "user",
            JsonApiType::Event => "event",
            JsonApiType::Rsvp => "rsvp",
            JsonApiType::Message => "message",
            JsonApiType::Login => "login",
            JsonApiType::Device => "device",
            JsonApiType::Friend => "friend",
            JsonApiType::UserIdList => "useridlist",
        };
        write!(f, "{}", printable)
    }
}

#[derive(Clone)]
#[derive(Serialize, Deserialize)]
enum JsonApiRelationType {
    ToMany,
    ToOne,
}


//#[derive(Clone)]
#[derive(Serialize)]
struct JsonApiObject {
    jsonapi_type: Option<JsonApiType>,
    id: Option<String>,
    attributes: Option<Map<String, Value>>,
    relationship_name: Option<String>, // if current object IS a relationship
    relationships: Option<HashMap<String, Vec<JsonApiObject>>>, //key is the name of the relationship, vec contains the relationships
    relation_type: JsonApiRelationType,
}

//impl Clone for JsonApiObject {
//    fn clone(&self) -> JsonApiObject { *self }
//}

impl JsonApiObject {
    pub fn new() -> JsonApiObject {
        JsonApiObject {
            jsonapi_type: None,
            id: None,
            attributes:None,
            relationship_name: None,
            relationships: None,
            relation_type: JsonApiRelationType::ToMany,
        }
    }

    fn add_relationship(&mut self, json_api_object: JsonApiObject)  {
        if json_api_object.relationship_name.is_none() { return }
        if json_api_object.id.is_none() { return }

        if self.relationships.is_none() {
            self.relationships = Some(HashMap::new());
        }

        let relationship_name: String;
        if let &Some(ref relname) = &json_api_object.relationship_name {
            relationship_name = relname.clone();
        } else {
            return
        }

        if let &mut Some(ref mut rels) = &mut self.relationships {
            let c = match rels.entry((relationship_name)) {
                Vacant(entry) => entry.insert(Vec::with_capacity(32768)),
                Occupied(entry) => entry.into_mut()
            };
            c.push(json_api_object);
        }

    }

    //["type": resourceType.rawValue, "id": id , "attributes": resourceAttributes ?? [:], "relationships": relationships()]
    pub fn serialize(&self) -> Result<String, Error> {
        let mut map = Map::new();
        map.insert(String::from("id"), serde_json::to_value(&self.id).unwrap());
        map.insert(String::from("type"), serde_json::to_value(&self.jsonapi_type).unwrap());
        //TODO: add attributes and relationships
        map.insert(String::from("attributes"), serde_json::to_value(&self.attributes).unwrap());
        map.insert(String::from("relationships"), serde_json::to_value(&self.relationships()).unwrap());
        let res = serde_json::to_string(&map);
        return res;
    }

    fn relationships(&self) -> Option<Map<String, Value>> {
        if let &Some(ref rels) = &self.relationships {
            let result = rels.iter().map( |rel_item|
                return ("aaa".to_string(), serde_json::to_value(&*rel_item.1).unwrap())
            );
            return Some(result.into_iter().collect::<serde_json::Map<String, Value>>());
        } else {
            return Some(Map::new());
        }

//        guard let relItems = relationItems else { return [:] }
//        return relItems.mapValues { self.relationsData($0) }
    }
}


fn main() {
    let mut json_root = JsonApiRoot::new();

    // Building event object
    let mut json_event_obj = JsonApiObject::new();
    json_event_obj.jsonapi_type = Some(JsonApiType::Event);
    json_event_obj.id = Some(String::from("2"));
    let mut event_attr_dic = Map::new();
    event_attr_dic.insert(String::from("name"), serde_json::to_value(String::from("Event is my name")).unwrap());
    json_event_obj.attributes = Some(event_attr_dic);

    // Adding owner object to the event
    let mut json_owner_obj = JsonApiObject::new();
    json_owner_obj.jsonapi_type = Some(JsonApiType::User);
    json_owner_obj.id = Some(String::from("1"));
    let mut owner_attr_dic = Map::new();
    owner_attr_dic.insert(String::from("name"), serde_json::to_value(String::from("Joel la malice")).unwrap());
    json_owner_obj.attributes = Some(owner_attr_dic);
    json_owner_obj.relation_type = JsonApiRelationType::ToOne;
    json_owner_obj.relationship_name = Some(String::from("owner"));
    println!("{:?}", &json_owner_obj.serialize());

    json_event_obj.add_relationship(json_owner_obj);

    // Adding event object to root
    json_root.add_root_item(json_event_obj);

    println!("{:?}", json_root.serialize());
}
