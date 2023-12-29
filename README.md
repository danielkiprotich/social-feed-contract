# Social Feed ICP

This is an Internet Computer canister that implements a blogging platform. Users can create posts, search for posts, view comments on a post, and interact with user accounts. The code is written in Rust and utilizes the Candid interface for interaction with the Internet Computer.

Here is a breakdown of the key components and functionalities:

1. **Type Aliases:**
   - `Memory`: Represents a virtual memory type, specifically `VirtualMemory<DefaultMemoryImpl>`.
   - `IdCell`: A cell for holding a `u64` value within the virtual memory.

2. **Struct Definitions:**
   - `Post`: Represents a blog post with fields such as `id`, `title`, `content`, `user_id`, `comments`, and `likes`.
   - `User`: Represents a user with fields like `id`, `name`, `phone`, `password`, and `post_ids`.
   - `Comment`: Represents a comment with fields `creator_id`, `post_id`, and `content`.

3. **Trait Implementations:**
   - `Storable` and `BoundedStorable` are implemented for `Post` and `User`. These traits define methods for converting the struct to and from bytes.

4. **Thread-Local Static Variables:**
   - `MEMORY_MANAGER`: Manages virtual memory for the canister.
   - `ID_COUNTER`: Tracks global IDs for posts and users.
   - `POST_STORAGE`: `StableBTreeMap` for storing posts.
   - `USER_STORAGE`: `StableBTreeMap` for storing users.

5. **Payload Structs:**
   - `UserPayload`, `PostPayload`, `EditPostPayload`, `AddComment`, `CommentPayload`, `EditUserPayload`, and `AccessPayload`: Data structures used in update functions to provide information for creating or updating users, posts, comments, etc.

6. **Update Functions:**
   - `add_post`: Creates a new post.
   - `get_all_posts`: Retrieves all posts.
   - `get_post_by_id`: Retrieves a specific post by ID.
   - `search_post_by_title`: Searches for posts by title.
   - `get_post_comments`: Retrieves comments for a specific post.
   - `edit_post`: Edits the content of a post.
   - `add_user`: Creates a new user.
   - `get_all_users`: Retrieves all users.
   - `get_user_by_name`: Retrieves users by name.
   - `get_user_by_id`: Retrieves a specific user by ID.
   - `edit_user`: Edits user information.
   - `comment_on_post`: Adds a comment to a post.

7. **Error Handling:**
   - `Error` enum: Represents different types of errors that can occur during execution, such as `NotFound`, `AlreadyInit`, `InvalidPayload`, and `Unauthorized`.

8. **Candid Interface Export:**
   - `ic_cdk::export_candid!()`: Generates the Candid interface for this canister.

It uses the `validator` crate for payload validation and the `serde` crate for serialization and deserialization. Additionally, the `ic_cdk` and `candid` crates are used for interacting with the Internet Computer platform and generating the Candid interface, respectively.

## ICP

To learn more before you start working with social_feeds, see the following documentation available online:

- [Quick Start](https://internetcomputer.org/docs/quickstart/quickstart-intro)
- [SDK Developer Tools](https://internetcomputer.org/docs/developers-guide/sdk-guide)
- [Rust Canister Devlopment Guide](https://internetcomputer.org/docs/rust-guide/rust-intro)
- [ic-cdk](https://docs.rs/ic-cdk)
- [ic-cdk-macros](https://docs.rs/ic-cdk-macros)
- [Candid Introduction](https://internetcomputer.org/docs/candid-guide/candid-intro)
- [JavaScript API Reference](https://erxue-5aaaa-aaaab-qaagq-cai.raw.icp0.io)

If you want to start working on your project right away, you might want to try the following commands:

```bash
cd social_feeds/
dfx help
dfx canister --help
```

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background

# Deploys your canisters to the replica and generates your candid interface
dfx deploy
```

Once the job completes, your application will be available at `http://localhost:4943?canisterId={asset_canister_id}`.

If you have made changes to your backend canister, you can generate a new candid interface with

```bash
npm run generate
```

at any time. This is recommended before starting the frontend development server, and will be run automatically any time you run `dfx deploy`.

If you are making frontend changes, you can start a development server with

```bash
npm start
```

Which will start a server at `http://localhost:8080`, proxying API requests to the replica at port 4943.

### Note on frontend environment variables

If you are hosting frontend code somewhere without using DFX, you may need to make one of the following adjustments to ensure your project does not fetch the root key in production:

- set`DFX_NETWORK` to `production` if you are using Webpack
- use your own preferred method to replace `process.env.DFX_NETWORK` in the autogenerated declarations
  - Setting `canisters -> {asset_canister_id} -> declarations -> env_override to a string` in `dfx.json` will replace `process.env.DFX_NETWORK` with the string in the autogenerated declarations
- Write your own `createActor` constructor
