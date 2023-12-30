#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::collections::HashMap;
use std::{borrow::Cow, cell::RefCell};
use validator::Validate;

// Define type aliases for convenience
type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Post {
    id: u64,
    title: String,
    content: String,
    user_id: u64,
    comments: HashMap<u64, Comment>,
    likes: u32,
}

// Implement the 'Storable' traits
impl Storable for Post {
    // Conversion to bytes
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    // Conversion from bytes
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct User {
    id: u64,
    name: String,
    phone: String,
    password: String,
    post_ids: Vec<u64>,
}

impl Storable for User {
    // Conversion to bytes
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    // Conversion from bytes
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Comment {
    creator_id: u64,
    post_id: u64,
    content: String,
}

// Implement the 'BoundedStorable' traits
impl BoundedStorable for Post {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl BoundedStorable for User {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// Define thread-local static variables for memory management and storage
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static POST_STORAGE: RefCell<StableBTreeMap<u64, Post, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    static USER_STORAGE: RefCell<StableBTreeMap<u64, User, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));
}

// Struct for payload date used in update functions
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default, Validate)]
struct UserPayload {
    #[validate(length(min = 3))]
    name: String,
    #[validate(length(min = 3))]
    phone: String,
    password: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default, Validate)]
struct PostPayload {
    #[validate(length(min = 3))]
    title: String,
    #[validate(length(min = 6))]
    content: String,
    user_id: u64,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct EditPostPayload {
    post_id: u64,
    user_id: u64,
    content: String,
    user_password: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct AddComment {
    post_id: u64,
    user_id: u64,
    comment: String,
    user_password: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default, Validate)]
struct CommentPayload {
    #[validate(length(min = 3))]
    name: String,
    user_id: u64,
    #[validate(length(min = 4))]
    password: String,
    user_password: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct EditUserPayload {
    user_id: u64,
    name: String,
    password: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct AccessPayload {
    comment_id: u64,
    post_id: u64,
    comment_password: String,
}

// Update function to add a post
#[ic_cdk::update]
fn add_post(payload: PostPayload) -> Result<Post, Error> {
    // validate payload
    let validate_payload = payload.validate();
    if validate_payload.is_err() {
        return Err(Error::InvalidPayload {
            msg: validate_payload.unwrap_err().to_string(),
        });
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    // check is user_id exist
    match USER_STORAGE.with(|users| users.borrow().get(&payload.user_id)) {
        Some(_) => {
            let post = Post {
                id,
                title: payload.title.clone(),
                content: payload.content,
                user_id: payload.user_id,
                comments: HashMap::new(),
                likes: 0,
            };

            match POST_STORAGE.with(|s| s.borrow_mut().insert(id, post.clone())) {
                None => Ok(post),
                Some(_) => Err(Error::InvalidPayload {
                    msg: format!("Could not add post title: {}", payload.title),
                }),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("user of id: {} not found", id),
        }),
    }
}

// get all posts
#[ic_cdk::query]
fn get_all_posts() -> Result<Vec<Post>, Error> {
    // Retrieve all Posts from the storage
    let post_map: Vec<(u64, Post)> = POST_STORAGE.with(|s| s.borrow().iter().collect());
    // Create a vector from tuple
    let posts: Vec<Post> = post_map.into_iter().map(|(_, post)| post).collect();

    match posts.len() {
        0 => Err(Error::NotFound {
            msg: format!("no Posts found, please add a post"),
        }),
        _ => Ok(posts),
    }
}

// query post by ID
#[ic_cdk::query]
fn get_post_by_id(id: u64) -> Result<Post, Error> {
    match POST_STORAGE.with(|posts| posts.borrow().get(&id)) {
        Some(post) => Ok(post),
        None => Err(Error::NotFound {
            msg: format!("post id:{} does not exist", id),
        }),
    }
}

// search post by title
#[ic_cdk::query]
fn search_post_by_title(search: String) -> Result<Vec<Post>, Error> {
    let query = search.to_lowercase();
    // Retrieve all Posts from the storage
    let post_map: Vec<(u64, Post)> = POST_STORAGE.with(|s| s.borrow().iter().collect());
    let posts: Vec<Post> = post_map.into_iter().map(|(_, post)| post).collect();

    // Filter the posts by title
    let incomplete_posts: Vec<Post> = posts
        .into_iter()
        .filter(|post| (post.title).to_lowercase().contains(&query))
        .collect();

    // Check if any posts are found
    match incomplete_posts.len() {
        0 => Err(Error::NotFound {
            msg: format!("No posts for search: {} could be found", query),
        }),
        _ => Ok(incomplete_posts),
    }
}

// get post by ID
#[ic_cdk::query]
fn get_post_comments(id: u64) -> Result<HashMap<u64, Comment>, Error> {
    match POST_STORAGE.with(|comments| comments.borrow().get(&id)) {
        Some(post) => match post.comments.len() {
            0 => Err(Error::NotFound {
                msg: format!("No comments found"),
            }),
            _ => Ok(post.comments),
        },
        None => Err(Error::NotFound {
            msg: format!("post id:{} does not exist", id),
        }),
    }
}

// update function to edit a post where authorizations is by password
#[ic_cdk::update]
fn edit_post(payload: EditPostPayload) -> Result<Post, Error> {
    let post = POST_STORAGE.with(|posts| posts.borrow().get(&payload.post_id));
    let user = USER_STORAGE.with(|user| user.borrow().get(&payload.user_id));

    match user {
        Some(user) => {
            // check if the password provided matches post
            if user.password != payload.user_password {
                return Err(Error::Unauthorized {
                    msg: format!("Unauthorized, password does not match, try again"),
                });
            }
            match post {
                Some(post) => {
                    // check if the password provided matches post
                    if user.password != payload.user_password {
                        return Err(Error::Unauthorized {
                            msg: format!("Unauthorized, password does not match, try again"),
                        });
                    }

                    let new_post = Post {
                        content: payload.content,
                        ..post.clone()
                    };

                    match POST_STORAGE.with(|s| s.borrow_mut().insert(post.id, new_post.clone())) {
                        Some(_) => Ok(new_post),
                        None => Err(Error::InvalidPayload {
                            msg: format!("Could not edit post name: {}", post.title),
                        }),
                    }
                }
                None => Err(Error::NotFound {
                    msg: format!("post of id: {} not found", payload.post_id),
                }),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("post of id: {} not found", payload.post_id),
        }),
    }
}

// Create new User
#[ic_cdk::update]
fn add_user(payload: UserPayload) -> Result<User, Error> {
    // validate payload
    let validate_payload = payload.validate();
    if validate_payload.is_err() {
        return Err(Error::InvalidPayload {
            msg: validate_payload.unwrap_err().to_string(),
        });
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    let user = User {
        id,
        name: payload.name.clone(),
        phone: payload.phone,
        password: payload.password,
        post_ids: vec![],
    };

    match USER_STORAGE.with(|s| s.borrow_mut().insert(id, user.clone())) {
        Some(_) => Err(Error::InvalidPayload {
            msg: format!("Could not add user name: {}", payload.name),
        }),
        None => Ok(user),
    }
}

// Query all users
#[ic_cdk::query]
fn get_all_users() -> Result<Vec<User>, Error> {
    // Retrieve all Users from the storage
    let user_map: Vec<(u64, User)> = USER_STORAGE.with(|s| s.borrow().iter().collect());
    // Create a vector from tuple
    let users: Vec<User> = user_map
        .into_iter()
        .map(|(_, user)| user)
        .map(|user| User {
            password: "-".to_string(),
            ..user
        })
        .collect();

    match users.len() {
        0 => Err(Error::NotFound {
            msg: format!("no Users found"),
        }),
        _ => Ok(users),
    }
}

// Get Users by name
#[ic_cdk::query]
fn get_user_by_name(search: String) -> Result<Vec<User>, Error> {
    let query = search.to_lowercase();
    // Retrieve all Users from the storage
    let user_map: Vec<(u64, User)> = USER_STORAGE.with(|s| s.borrow().iter().collect());
    let users: Vec<User> = user_map.into_iter().map(|(_, user)| user).collect();

    // Filter the users by name
    let incomplete_posts: Vec<User> = users
        .into_iter()
        .filter(|user| (user.name).to_lowercase().contains(&query))
        .map(|user| User {
            password: "-".to_string(),
            ..user
        })
        .collect();

    // Check if any users are found
    match incomplete_posts.len() {
        0 => Err(Error::NotFound {
            msg: format!("No users for name: {} could be found", query),
        }),
        _ => Ok(incomplete_posts),
    }
}

// get user by ID
#[ic_cdk::query]
fn get_user_by_id(id: u64) -> Result<User, Error> {
    match USER_STORAGE.with(|users| users.borrow().get(&id)) {
        Some(user) => Ok(User {
            password: "-".to_string(),
            ..user
        }),
        None => Err(Error::NotFound {
            msg: format!("user of id: {} not found", id),
        }),
    }
}

// update function to edit a user where only owners of users can edit title, is_community, price and description. Non owners can only edit descriptions of communtiy users. authorizations is by password
#[ic_cdk::update]
fn edit_user(payload: EditUserPayload) -> Result<User, Error> {
    let user = USER_STORAGE.with(|users| users.borrow().get(&payload.user_id));

    match user {
        Some(user) => {
            // check if the password provided matches user
            if user.password != payload.password {
                return Err(Error::Unauthorized {
                    msg: format!("Unauthorized, password does not match, try again"),
                });
            }

            let new_user = User {
                name: payload.name,
                ..user.clone()
            };

            match USER_STORAGE.with(|s| s.borrow_mut().insert(user.id, new_user.clone())) {
                Some(_) => Ok(new_user),
                None => Err(Error::InvalidPayload {
                    msg: format!("Could not edit user title: {}", user.name),
                }),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("user of id: {} not found", payload.user_id),
        }),
    }
}

// function to comment on a post
#[ic_cdk::update]
fn comment_on_post(payload: AddComment) -> Result<String, Error> {
    // get post
    let post = POST_STORAGE.with(|post| post.borrow().get(&payload.post_id));
    let user = USER_STORAGE.with(|user| user.borrow().get(&payload.user_id));

    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    match user {
        Some(user) => {
            if user.password != payload.user_password {
                return Err(Error::Unauthorized {
                    msg: format!("Comment Access unauthorized, password does not match, try again"),
                });
            }
            match post {
                Some(post) => {
                    let mut new_comment_post_ids = user.post_ids.clone();
                    new_comment_post_ids.push(post.id);
                    let new_comment = Comment {
                        creator_id: payload.user_id,
                        post_id: payload.post_id,
                        content: payload.comment,
                    };

                    let mut new_post = post.clone();
                    // add comment to post
                    new_post.comments.insert(id, new_comment);
                    
                    match POST_STORAGE.with(|s| s.borrow_mut().insert(post.id, new_post.clone())) {
                        Some(_) => Ok(format!("succesfully added comment")),
                        None => Err(Error::InvalidPayload {
                            msg: format!("Could not update comment"),
                        }),
                    }
                }
                None => Err(Error::NotFound {
                    msg: format!("Post of id: {} not found", payload.post_id),
                }),
            }
        }
        None => Err(Error::NotFound {
            msg: format!("user of id: {} not found", payload.user_id),
        }),
    }
}

// Define an Error enum for handling errors
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    AlreadyInit { msg: String },
    InvalidPayload { msg: String },
    Unauthorized { msg: String },
}

// Candid generator for exporting the Candid interface
ic_cdk::export_candid!();
