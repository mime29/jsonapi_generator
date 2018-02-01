extern crate serde;

extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use serde_json::Error;
use serde_json::Map;
use serde_json::Value;
use std::fmt::{self};

use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use std::collections::hash_map::Entry::{Occupied, Vacant};

macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::serde_json::Map::new();  //::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
     };
);

//#[derive(Serialize, Deserialize)]
struct JsonApiRoot {
    jsonapi: Option<Map<String, Value>>,
    links: Option<Map<String, Value>>,
    meta: Option<Map<String, Value>>,
    root_items: Option<Vec<JsonApiObject>>,
}

impl JsonApiRoot {
    pub fn new() -> JsonApiRoot {
        JsonApiRoot {
            jsonapi:Some(map!{"version".to_string() => "1.0".to_string().into()}),
            links: Some(Map::new()),
            meta: Some(Map::new()),
            root_items: None,
        }
    }

    fn add_root_item(&mut self, json_api_object: JsonApiObject) {
        if let Some(ref mut rels) = self.root_items {
            rels.push(json_api_object);
            println!("root item added")
        } else {
            self.root_items = Some(vec![json_api_object])
        }
    }

    pub fn serialize(self) -> Result<String, Error> {
        let mut json_api_root = Map::new();
        json_api_root.insert(String::from("data"), self.data());
        json_api_root.insert(String::from("included"), self.included());
        json_api_root.insert(String::from("links"), self.links());
        json_api_root.insert(String::from("meta"), self.meta());
        json_api_root.insert(String::from("jsonapi"), self.jsonapi());
        let res = serde_json::to_string(&json_api_root);
        return res;
    }

    fn data(&self) -> Value {
        if let &Some(ref r_items) = &self.root_items {
            //TODO: Return vec of data if the endpoint is plural (ex: api/events)
            if &r_items.len() == &1 {
                println!("only one root item found");
                let json_api_obj = r_items.first().unwrap().to_map();
                return serde_json::to_value(json_api_obj).unwrap()
            } else {
                println!("many root items found");
                let vec_of_json_obj = r_items.iter().map(|item| item.to_map()).collect::<Vec<_>>();
                return serde_json::to_value(vec_of_json_obj).unwrap()
            }
        }
        return Map::new().into();
    }


    // We create a Set from a Vector to remove unique items
    // This uses Eq and Hash on JsonApiObject
    fn build_unique_relationships(&self) -> HashSet<&JsonApiObject> {
        let all_relationships = match self.root_items {
            Some(ref rootitems) => rootitems.iter().flat_map(|item| match &item.relationships {
                    &Some(ref rels) => rels.iter().flat_map(|rel| rel.1).collect(),
                    &None => HashSet::new(),
                }).collect(),
            None => HashSet::new(),
        };
        all_relationships
    }

    fn included(&self) -> Value {
        let unique_relationships = &self.build_unique_relationships();
        let vec_of_rels = unique_relationships.iter().map(|rel| rel.to_map()).collect::<Vec<_>>();
        return serde_json::to_value(vec_of_rels).unwrap();
    }

    fn links(&self) -> Value {
        return serde_json::to_value(&self.links).unwrap();
    }

    fn meta(&self) -> Value {
        return serde_json::to_value(&self.meta).unwrap();
    }

    fn jsonapi(&self) -> Value {
        return serde_json::to_value(&self.jsonapi).unwrap();
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

impl std::cmp::PartialEq for JsonApiObject {
    fn eq(&self, other: &JsonApiObject) -> bool {
        if let &Some(ref selftype) = &self.jsonapi_type {
            if let &Some(ref othertype) = &other.jsonapi_type {
                return selftype.to_string() == othertype.to_string() && self.id == other.id
            }
        }
        return false
    }
}
impl Eq for JsonApiObject {}

impl Hash for JsonApiObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if let (&Some(ref id), &Some(ref jsonapi_type)) = (&self.id, &self.jsonapi_type) {
            id.hash(state);
            jsonapi_type.to_string().hash(state);
        }
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
            println!("relationship has been pushed into map");
        }

    }

    fn to_map(&self) -> Map<String, Value> {
        let mut map = Map::new();
        map.insert(String::from("id"), serde_json::to_value(&self.id).unwrap());
        let json_type = &self.jsonapi_type.as_ref().unwrap();
        map.insert(String::from("type"), serde_json::to_value(json_type.to_string().clone()).unwrap());
        map.insert(String::from("attributes"), serde_json::to_value(&self.attributes).unwrap());
        map.insert(String::from("relationships"), serde_json::to_value(&self.relationships()).unwrap());
        map
    }

    //["type": resourceType.rawValue, "id": id , "attributes": resourceAttributes ?? [:], "relationships": relationships()]
//    pub fn serialize(&self) -> Result<String, Error> {
//        let map = self.to_map();
//        let res = serde_json::to_string(&map);
//        return res;
//    }

    fn relationships(&self) -> Option<Map<String, Value>> {
        if let &Some(ref rels) = &self.relationships {
            let result = rels.iter().map( |rel_item|
                {
                    let rel_name: String = rel_item.0.clone();
                    let rel_vec_of_map = &rel_item.1.iter().map(|json_obj|
                        json_obj.to_map()
                    ).collect::<Vec<_>>();
                    return (rel_name, serde_json::to_value(rel_vec_of_map).unwrap())
                }
            );
            println!("value present");
            return Some(result.into_iter().collect::<serde_json::Map<String, Value>>());
        } else {
            eprintln!("value not present");
            return Some(Map::new());
        }
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
    //println!("{:?}", &json_owner_obj.serialize());

    json_event_obj.add_relationship(json_owner_obj);

    // Adding event object to root
    json_root.add_root_item(json_event_obj);

    println!("{:?}", json_root.serialize());
}
