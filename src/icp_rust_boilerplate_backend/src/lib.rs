#[macro_use]
extern crate serde;
use candid::{CandidType, Decode, Encode};
use ic_cdk::api::{time, call, storage, request};
use ic_cdk::export::candid::{CandidMethod, candid_method};
use ic_cdk::export::pandora::Read;
use ic_cdk::export::Principal;
use ic_cdk_macros::{query, update};
use ic_nns_constants::GOVERNANCE_CANISTER_ID;
use ic_protobuf::registry::subnet::v1::{SubnetListRecord, SubnetListRecordOrBuilder};
use ic_stable_storage::{StableMemory, StableVec};
use std::{borrow::Cow, cell::RefCell, collections::BTreeMap, str::FromStr};

type ID = u64;
type TimeStamp = u64;

#[derive(
    Copy, Clone, PartialEq, Eq, Hash, Debug, CandidType, Serialize, Deserialize,
)]
enum BusinessType {
    #[serde(rename = "restaurant")]
    Restaurant,
    #[serde(rename = "grocery")]
    Grocery,
    #[serde(rename = "bakery")]
    Bakery,
    #[serde(rename = "other")]
    Other,
}

#[derive(
    Copy, Clone, PartialEq, Eq, Hash, Debug, CandidType, Serialize, Deserialize,
)]
enum FoodType {
    #[serde(rename = "vegetables")]
    Vegetables,
    #[serde(rename = "fruits")]
    Fruits,
    #[serde(rename = "dairy")]
    Dairy,
    #[serde(rename = "meat")]
    Meat,
    #[serde(rename = "grains")]
    Grains,
    #[serde(rename = "bakery")]
    Bakery,
    #[serde(rename = "beverages")]
    Beverages,
    #[serde(rename = "other")]
    Other,
}

#[derive(CandidType, Clone, Serialize, Deserialize)]
struct SurplusPost {
    id: ID,
    donor_id: Principal,
    food_type: FoodType,
    quantity_kg: u32,
    best_before_date: TimeStamp,
    handling_instructions: String,
    assigned: bool,
}

#[derive(CandidType, Clone, Serialize, Deserialize)]
struct DonorProfile {
    id: ID,
    name: String,
    phone_number: String,
    email: String,
    address: String,
    business_type: BusinessType,
    created_at: TimeStamp,
}

#[derive(CandidType, Clone, Serialize, Deserialize)]
struct ReceiverProfile {
    id: ID,
    name: String,
    phone_number: String,
    email: String,
    address: String,
    created_at: TimeStamp,
}

#[derive(CandidType, Clone, Serialize, Deserialize)]
struct DriverProfile {
    id: ID,
    name: String,
    phone_number: String,
    email: String,
    address: String,
    created_at: TimeStamp,
}

#[derive(CandidType, Clone, Serialize, Deserialize)]
struct Assignment {
    id: ID,
    receiver_id: ID,
    surplus_post_id: ID,
    driver_id: ID,
    status: String,
    created_at: TimeStamp,
}

#[derive(CandidType, Clone, Serialize, Deserialize)]
struct SurplusRecord {
    id: ID,
    surplus_post_id: ID,
    driver_id: ID,
    delivered_at: TimeStamp,
    rating: Option<u8>,
}

// Using StableMemory and StableVec for storing data in the IC's stable memory 
// allows efficient, version-controlled storage of arbitrarily complex data structures.
lazy_static! {
    static ref DONORS: StableVec<DonorProfile> = StableVec::new("donors");
    static ref RECEIVERS: StableVec<ReceiverProfile> = StableVec::new("receivers");
    static ref DRIVERS: StableVec<DriverProfile> = StableVec::new("drivers");
    static ref SURPLUS_POSTS: StableVec<SurplusPost> = StableVec::new("surplus_posts");
    static ref ASSIGNMENTS: StableVec<Assignment> = StableVec::new("assignments");
    static ref SURPLUS_RECORDS: StableVec<SurplusRecord> = StableVec::new("surplus_records");
    static ref SLR: StableMemory<SubnetListRecord> =
        StableMemory::new("subnets_list_record");
}

// Thread-local storage allows runtime-efficient access to data that isn't frequently changing.
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<u64> = RefCell::new(
        0
    );
}

// Function to create a new donor profile
#[update]
async fn create_donor_profile(
    name: String,
    phone_number: String,
    email: String,
    address: String,
    business_type: BusinessType,
) -> Result<DonorProfile, String> {
    let sender = request::caller();

    // Check if sender is authorized to create a donor profile
    if !is_governance_accepted(sender).await {
        return Err("Unauthorized".into());
    }

    // Ensure required fields are present
    if name.is_empty() || phone_number.is_empty() || email.is_empty() || address.is_empty() {
        return Err("All fields are required".into());
    }

    // Ensure a unique email was provided
    if DONORS.read().unwrap().iter().any(|donor| donor.email == email) {
        return Err("Email already exists".into());
    }

    // Ensure a valid email format was provided
    if !email.contains('@') {
        return Err("Invalid email format".into());
    }

    // Ensure a valid phone number was provided
    if phone_number.len() != 10 || !phone_number.chars().all(|c| c.is_numeric()) {
        return Err("Invalid phone number format".into());
    }

    let id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow();
        *counter.borrow_mut() = current_value + 1;
        current_value + 1
    });

    // Create and insert the new donor profile
    let donor = DonorProfile {
        id,
        name,
        phone_number,
        email,
        address,
        business_type,
        created_at: time(),
    };
    DONORS.write().unwrap().push(donor.clone());

    Ok(donor)
}

// Function to create a new receiver profile
#[update]
async fn create_receiver_profile(
    name: String,
    phone_number: String,
    email: String,
    address: String,
) -> Result<ReceiverProfile, String> {
    let sender = request::caller();

    // Check if sender is authorized to create a receiver profile
    if !is_governance_accepted(sender).await {
        return Err("Unauthorized".into());
    }

    // Ensure required fields are present
    if name.is_empty() || phone_number.is_empty() || email.is_empty() || address.is_empty() {
        return Err("All fields are required".into());
    }

    // Ensure a unique email was provided
    if RECEIVERS.read().unwrap().iter().any(|receiver| receiver.email == email) {
        return Err("Email already exists".into());
    }

    // Ensure a valid email format was provided
    if !email.contains('@') {
        return Err("Invalid email format".into());
    }

    // Ensure a valid phone number was provided
    if phone_number.len() != 10 || !phone_number.chars().all(|c| c.is_numeric()) {
        return Err("Invalid phone number format".into());
    }

    let id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow();
        *counter.borrow_mut() = current_value + 1;
        current_value + 1
    });

    // Create and insert the new receiver profile
    let receiver = ReceiverProfile {
        id,
        name,
        phone_number,
        email,
        address,
        created_at: time(),
    };
    RECEIVERS.write().unwrap().push(receiver.clone());

    Ok(receiver)
}

// Function to create a new driver profile
#[update]
async fn create_driver_profile(
    name: String,
    phone_number: String,
    email: String,
    address: String,
) -> Result<DriverProfile, String> {
    let sender = request::caller();

    // Check if sender is authorized to create a driver profile
    if !is_governance_accepted(sender).await {
        return Err("Unauthorized".into());
    }

    // Ensure required fields are present
    if name.is_empty() || phone_number.is_empty() || email.is_empty() || address.is_empty() {
        return Err("All fields are required".into());
    }

    // Ensure a unique email was provided
    if DRIVERS.read().unwrap().iter().any(|driver| driver.email == email) {
        return Err("Email already exists".into());
    }

    // Ensure a valid email format was provided
    if !email.contains('@') {
        return Err("Invalid email format".into());
    }

    // Ensure a valid phone number was provided
    if phone_number.len() != 10 || !phone_number.chars().all(|c| c.is_numeric()) {
        return Err("Invalid phone number format".into());
    }

    let id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow();
        *counter.borrow_mut() = current_value + 1;
        current_value + 1
    });

    // Create and insert the new driver profile
    let driver = DriverProfile {
        id,
        name,
        phone_number,
        email,
        address,
        created_at: time(),
    };
    DRIVERS.write().unwrap().push(driver.clone());

    Ok(driver)
}

// Function to create a new surplus post
#[update]
async fn create_surplus_post(
    donor_id: Principal,
    food_type: FoodType,
    quantity_kg: u32,
    best_before_date: TimeStamp,
    handling_instructions: String,
) -> Result<SurplusPost, String> {
    let sender = request::caller();

    // Check if sender is authorized to create a surplus post
    if sender != donor_id && !is_governance_accepted(sender).await {
        return Err("Unauthorized".into());
    }

    // Ensure required fields are present
    if donor_id == Principal::anonymous() || quantity_kg == 0 || best_before_date == 0 {
        return Err("All fields are required".into());
    }

    let donor_exists = DONORS
        .read()
        .unwrap()
        .iter()
        .any(|donor| donor.id == donor_id);
    if !donor_exists {
        return Err("Donor ID does not exist".into());
    }

    let id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow();
        *counter.borrow_mut() = current_value + 1;
        current_value + 1
    });

    // Create and insert the new surplus post
    let surplus_post = SurplusPost {
        id,
        donor_id,
        food_type,
        quantity_kg,
        best_before_date,
        handling_instructions,
        assigned: false,
    };
    SURPLUS_POSTS.write().unwrap().push(surplus_post.clone());

    Ok(surplus_post)
}

// Function to create a new assignment
#[update]
async fn create_assignment(
    receiver_id: ID,
    surplus_post_id: ID,
    driver_id: ID,
) -> Result<Assignment, String> {
    let sender = request::caller();

    // Check if sender is authorized to create an assignment
    if sender != driver_id && !is_governance_accepted(sender).await {
        return Err("Unauthorized".into());
    }

    // Ensure all required fields are present
    if receiver_id == 0 || surplus_post_id == 0 || driver_id == 0 {
        return Err("All fields are required".into());
    }

    // Ensure the associated profile with each ID exists
    let receiver_exists = RECEIVERS
        .read()
        .unwrap()
        .iter()
        .any(|receiver| receiver.id == receiver_id);
    let surplus_post_exists = SURPLUS_POSTS
        .read()
        .unwrap()
        .iter()
        .any(|post| post.id == surplus_post_id);
    let driver_exists = DRIVERS
        .read()
        .unwrap()
        .iter()
        .any(|driver| driver.id == driver_id);
    if !receiver_exists || !surplus_post_exists || !driver_exists {
        return Err("The provided IDs are invalid.".into());
    }

    let already_assigned = ASSIGNMENTS
        .read()
        .unwrap()
        .iter()
        .any(|assignment| assignment.surplus_post_id == surplus_post_id);
    if already_assigned {
        return Err("The surplus post is already assigned.".into());
    }

    let driver_assigned = ASSIGNMENTS
        .read()
        .unwrap()
        .iter()
        .any(|assignment| assignment.driver_id == driver_id);
    if driver_assigned {
        return Err("The driver is already assigned to another post.".into());
    }

    let id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow();
        *counter.borrow_mut() = current_value + 1;
        current_value + 1
    });

    // Create the new assignment and insert it into storage
    let assignment = Assignment {
        id,
        receiver_id,
        surplus_post_id,
        driver_id,
        status: "Pending".into(),
        created_at: time(),
    };
    ASSIGNMENTS.write().unwrap().push(assignment.clone());

    Ok(assignment)
}

// Function to create a new surplus record
#[update]
async fn create_surplus_record(
    surplus_post_id: ID,
    driver_id: ID,
    rating: Option<u8>,
) -> Result<SurplusRecord, String> {
    let sender = request::caller();

    // Check if sender is authorized to create a surplus record
    if sender != driver_id && !is_governance_accepted(sender).await {
        return Err("Unauthorized".into());
    }

    // Ensure all required fields are present
    if surplus_post_id == 0 || driver_id == 0 {
        return Err("All fields are required".into());
    }

    // Ensure the associated profile with each ID exists
    let surplus_post_exists = SURPLUS_POSTS
        .read()
        .unwrap()
        .iter()
        .any(|post| post.id == surplus_post_id && post.assigned);
    let driver_exists = DRIVERS
        .read()
        .unwrap()
        .iter()
        .any(|driver| driver.id == driver_id);
    if !surplus_post_exists || !driver_exists {
        return Err("The provided IDs are invalid.".into());
    }

    let already_recorded = SURPLUS_RECORDS
        .read()
        .unwrap()
        .iter()
        .any(|record| record.surplus_post_id == surplus_post_id);
    if already_recorded {
        return Err("The surplus post is already recorded for delivery.".into());
    }

    let id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow();
        *counter.borrow_mut() = current_value + 1;
        current_value + 1
    });

    // Create and insert the new surplus record
    let surplus_record = SurplusRecord {
        id,
        surplus_post_id,
        driver_id,
        delivered_at: time(),
        rating,
    };
    SURPLUS_RECORDS.write().unwrap().push(surplus_record.clone());

    Ok(surplus_record)
}

// Function to get all donor profiles
#[query]
async fn get_all_donors() -> Vec<DonorProfile> {
    DONORS.read().unwrap().to_vec()
}

// Function to get all receiver profiles
#[query]
async fn get_all_receivers() -> Vec<ReceiverProfile> {
    RECEIVERS.read().unwrap().to_vec()
}

// Function to get all driver profiles
#[query]
async fn get_all_drivers() -> Vec<DriverProfile> {
    DRIVERS.read().unwrap().to_vec()
}

// Function to get all surplus posts
#[query]
async fn get_all_surplus_posts() -> Vec<SurplusPost> {
    SURPLUS_POSTS.read().unwrap().to_vec()
}

// Function to get all assignments
#[query]
async fn get_all_assignments() -> Vec<Assignment> {
    ASSIGNMENTS.read().unwrap().to_vec()
}

// Function to get all surplus records
#[query]
async fn get_all_surplus_records() -> Vec<SurplusRecord> {
    SURPLUS_RECORDS.read().unwrap().to_vec()
}

// Check if the caller is authorized by the governance to perform actions on the canister
async fn is_governance_accepted(sender: Principal) -> bool {
    let governance = Principal::from_str(GOVERNANCE_CANISTER_ID).unwrap();
    let response: bool = call::call(
        governance,
        "canister_status_accepted_caller",
        (sender,),
    )
    .await
    .unwrap_or(false);
    response
}//=================================================================================================
// Entry point functions
//=================================================================================================

// Function to create a new donor profile
#[update]
async fn create_donor_profile(name: String, address: String, phone: String, email: String) -> Result<(), String> {
    let caller = caller();
    if !is_governance_accepted(caller).await {
        return Err("You are not authorized to perform this action.".into());
    }

    let id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow();
        *counter.borrow_mut() = current_value + 1;
        current_value + 1
    });

    let donor_profile = DonorProfile {
        id,
        name,
        address,
        phone,
        email,
    };
    DONORS.write().unwrap().push(donor_profile);
    Ok(())
}

// Function to create a new receiver profile
#[update]
async fn create_receiver_profile(name: String, address: String, phone: String, email: String) -> Result<(), String> {
    let caller = caller();
    if !is_governance_accepted(caller).await {
        return Err("You are not authorized to perform this action.".into());
    }

    let id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow();
        *counter.borrow_mut() = current_value + 1;
        current_value + 1
    });

    let receiver_profile = ReceiverProfile {
        id,
        name,
        address,
        phone,
        email,
    };
    RECEIVERS.write().unwrap().push(receiver_profile);
    Ok(())
}

// Function to create a new driver profile
#[update]
async fn create_driver_profile(name: String, address: String, phone: String, email: String) -> Result<(), String> {
    let caller = caller();
    if !is_governance_accepted(caller).await {
        return Err("You are not authorized to perform this action.".into());
    }

    let id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow();
        *counter.borrow_mut() = current_value + 1;
        current_value + 1
    });

    let driver_profile = DriverProfile {
        id,
        name,
        address,
        phone,
        email,
    };
    DRIVERS.write().unwrap().push(driver_profile);
    Ok(())
}

// Function to create a new surplus post
#[update]
async fn create_surplus_post(donor_id: u32, description: String, quantity: u32) -> Result<SurplusPost, String> {
    let caller = caller();
    if !is_governance_accepted(caller).await {
        return Err("You are not authorized to perform this action.".into());
    }

    // Check if the provided donor ID is valid
    let donor_exists = DONORS
        .read()
        .unwrap()
        .iter()
        .any(|donor| donor.id == donor_id);
    if !donor_exists {
        return Err("The provided donor ID is invalid.".into());
    }

    let id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow();
        *counter.borrow_mut() = current_value + 1;
        current_value + 1
    });

    let surplus_post = SurplusPost {
        id,
        donor_id,
        description,
        quantity,
        assigned: false,
        assigned_to: None,
    };
    SURPLUS_POSTS.write().unwrap().push(surplus_post.clone());
    Ok(surplus_post)
}

// Function to assign a driver to a surplus post for delivery
#[update]
async fn assign_driver_to_surplus_post(surplus_post_id: u32, driver_id: u32) -> Result<Assignment, String> {
    let caller = caller();
    if !is_governance_accepted(caller).await {
        return Err("You are not authorized to perform this action.".into());
    }

    // Check if the provided IDs and data are valid
    let driver_exists = DRIVERS
        .read()
        .unwrap()
        .iter()
        .any(|driver| driver.id == driver_id);
    let surplus_post_exists = SURPLUS_POSTS
        .read()
        .unwrap()
        .iter()
        .any(|post| post.id == surplus_post_id && !post.assigned);
    if !driver_exists || !surplus_post_exists {
        return Err("The provided IDs are invalid or the surplus post is already assigned.".into());
    }

    let id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow();
        *counter.borrow_mut() = current_value + 1;
        current_value + 1
    });

    let assignment = Assignment {
        id,
        driver_id,
        surplus_post_id,
        assigned_at: time(),
    };
    SURPLUS_POSTS
        .write()
        .unwrap()
        .iter_mut()
        .find(|post| post.id == surplus_post_id)
        .unwrap()
        .assigned = true;
    SURPLUS_POSTS
        .write()
        .unwrap()
        .iter_mut()
        .find(|post| post.id == surplus_post_id)
        .unwrap()
        .assigned_to = Some(driver_id);
    ASSIGNMENTS.write().unwrap().push(assignment.clone());
    Ok(assignment)
}

// Function to record the delivery of a surplus post by a driver
#[update]
async fn record_surplus_delivery(surplus_post_id: u32, driver_id: u32, rating: u32) -> Result<SurplusRecord, String> {
    let caller = caller();
    if !is_governance_accepted(caller).await {
        return Err("You are not authorized to perform this action.".into());
    }

    // Check if the provided IDs and data are valid
    let driver_exists = DRIVERS
        .read()
        .unwrap()
        .iter()
        .any(|driver| driver.id == driver_id);
    let surplus_post_exists = SURPLUS_POSTS
        .read()
        .unwrap()
        .iter()
        .any(|post| post.id == surplus_post_id && post.assigned);
    if !driver_exists || !surplus_post_exists {
        return Err("The provided IDs are invalid or the surplus post is not assigned.".into());
    }

    let id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow();
        *counter.borrow_mut() = current_value + 1;
        current_value + 1
    });

    let surplus_record = SurplusRecord {
        id,
        surplus_post_id,
        driver_id,
        delivered_at: time(),
        rating,
    };
    SURPLUS_POSTS
        .write()
        .unwrap()
        .iter_mut()
        .find(|post| post.id == surplus_post_id)
        .unwrap()
        .assigned = false;
    SURPLUS_POSTS
        .write()
        .unwrap()
        .iter_mut()
        .find(|post| post.id == surplus_post_id)
        .unwrap()
        .assigned_to = None;
    SURPLUS_RECORDS.write().unwrap().push(surplus_record.clone());
    Ok(surplus_record)
}

//=================================================================================================
// Internal helper functions
//=================================================================================================

// Function to get the current time in seconds since UNIX epoch
fn time() -> u64 {
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    u64::from(time)
}

// Function to get the address of the caller
fn caller() -> Principal {
    let user_id = ic_cdk::caller();
    user_id
}// Function to check if the caller is a governance canister
async fn is_governance_accepted(caller: Principal) -> bool {
    let governance = ic_cdk::id();
    governance == caller
}

//=================================================================================================
// Data structures
//=================================================================================================

// DonorProfile is a struct that represents a donor profile
#[derive(Clone, Debug, CandidType)]
struct DonorProfile {
    id: u32,
    name: String,
    address: String,
    phone: String,
    email: String,
}

// ReceiverProfile is a struct that represents a receiver profile
#[derive(Clone, Debug, CandidType)]
struct ReceiverProfile {
    id: u32,
    name: String,
    address: String,
    phone: String,
    email: String,
}

// DriverProfile is a struct that represents a driver profile
#[derive(Clone, Debug, CandidType)]
struct DriverProfile {
    id: u32,
    name: String,
    address: String,
    phone: String,
    email: String,
}

// SurplusPost is a struct that represents a surplus food post
#[derive(Clone, Debug, CandidType)]
struct SurplusPost {
    id: u32,
    donor_id: u32,
    description: String,
    quantity: u32,
    assigned: bool,
    assigned_to: Option<u32>,
}

// Assignment is a struct that represents an assignment of a driver to a surplus post
#[derive(Clone, Debug, CandidType)]
struct Assignment {
    id: u32,
    driver_id: u32,
    surplus_post_id: u32,
    assigned_at: u64,
}

// SurplusRecord is a struct that represents a record of a delivered surplus post by a driver
#[derive(Clone, Debug, CandidType)]
struct SurplusRecord {
    id: u32,
    surplus_post_id: u32,
    driver_id: u32,
    delivered_at: u64,
    rating: u32,
}

//=================================================================================================
// Global variables and constants
//=================================================================================================

// ID_COUNTER is a thread-local counter for generating unique IDs for each profile and post
thread_local!(static ID_COUNTER: RefCell<u32> = RefCell::new(0));

// DONORS is a global variable that holds all the donor profiles
static DONORS: Lazy<RwLock<Vec<DonorProfile>>> = Lazy::new(|| RwLock::new(Vec::new()));

// RECEIVERS is a global variable that holds all the receiver profiles
static RECEIVERS: Lazy<RwLock<Vec<ReceiverProfile>>> = Lazy::new(|| RwLock::new(Vec::new()));

// DRIVERS is a global variable that holds all the driver profiles
static DRIVERS: Lazy<RwLock<Vec<DriverProfile>>> = Lazy::new(|| RwLock::new(Vec::new()));

// SURPLUS_POSTS is a global variable that holds all the surplus food posts
static SURPLUS_POSTS: Lazy<RwLock<Vec<SurplusPost>>> = Lazy::new(|| RwLock::new(Vec::new()));

// ASSIGNMENTS is a global variable that holds all the assignments of drivers to surplus posts
static ASSIGNMENTS: Lazy<RwLock<Vec<Assignment>>> = Lazy::new(|| RwLock::new(Vec::new()));

// SURPLUS_RECORDS is a global variable that holds all the records of delivered surplus posts by drivers
static SURPLUS_RECORDS: Lazy<RwLock<Vec<SurplusRecord>>> = Lazy::new(|| RwLock::new(Vec::new()));//=================================================================================================
// Donor functions
//=================================================================================================

// Function to add a new donor profile
#[update]
async fn add_donor_profile(name: String, address: String, phone: String, email: String) -> Result<(), String> {
    let caller = ic_cdk::caller();
    if !is_governance_accepted(caller).await {
        return Err(String::from("Unauthorized access"));
    }
    
    // Generate a unique ID for the donor profile
    let id = ID_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        *counter += 1;
        *counter
    });
    
    // Create a new donor profile and add it to the DONORS global variable
    let donor = DonorProfile { id, name, address, phone, email };
    DONORS.write().await.push(donor);
    
    Ok(())
}

// Function to get all the donor profiles
#[query]
async fn get_donor_profiles() -> Vec<DonorProfile> {
    let donors = DONORS.read().await;
    donors.clone()
}

//=================================================================================================
// Receiver functions
//=================================================================================================

// Function to add a new receiver profile
#[update]
async fn add_receiver_profile(name: String, address: String, phone: String, email: String) -> Result<(), String> {
    let caller = ic_cdk::caller();
    if !is_governance_accepted(caller).await {
        return Err(String::from("Unauthorized access"));
    }
    
    // Generate a unique ID for the receiver profile
    let id = ID_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        *counter += 1;
        *counter
    });
    
    // Create a new receiver profile and add it to the RECEIVERS global variable
    let receiver = ReceiverProfile { id, name, address, phone, email };
    RECEIVERS.write().await.push(receiver);
    
    Ok(())
}

// Function to get all the receiver profiles
#[query]
async fn get_receiver_profiles() -> Vec<ReceiverProfile> {
    let receivers = RECEIVERS.read().await;
    receivers.clone()
}

//=================================================================================================
// Driver functions
//=================================================================================================

// Function to add a new driver profile
#[update]
async fn add_driver_profile(name: String, address: String, phone: String, email: String) -> Result<(), String> {
    let caller = ic_cdk::caller();
    if !is_governance_accepted(caller).await {
        return Err(String::from("Unauthorized access"));
    }
    
    // Generate a unique ID for the driver profile
    let id = ID_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        *counter += 1;
        *counter
    });
    
    // Create a new driver profile and add it to the DRIVERS global variable
    let driver = DriverProfile { id, name, address, phone, email };
    DRIVERS.write().await.push(driver);
    
    Ok(())
}

// Function to get all the driver profiles
#[query]
async fn get_driver_profiles() -> Vec<DriverProfile> {
    let drivers = DRIVERS.read().await;
    drivers.clone()
}

//=================================================================================================
// Surplus food post functions
//=================================================================================================

// Function to add a new surplus food post
#[update]
async fn add_surplus_post(donor_id: u32, description: String, quantity: u32) -> Result<(), String> {
    // Check if the donor profile exists
    let donors = DONORS.read().await;
    let donor_index = donors.iter().position(|d| d.id == donor_id);
    if donor_index.is_none() {
        return Err(String::from("Donor profile does not exist"));
    }
    
    // Generate a unique ID for the surplus post
    let id = ID_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        *counter += 1;
        *counter
    });
    
    // Create a new surplus post and add it to the SURPLUS_POSTS global variable
    let surplus_post = SurplusPost { id, donor_id, description, quantity, assigned: false, assigned_to: None };
    SURPLUS_POSTS.write().await.push(surplus_post);
    
    Ok(())
}

// Function to get all the surplus food posts
#[query]
async fn get_surplus_posts() -> Vec<SurplusPost> {
    let surplus_posts = SURPLUS_POSTS.read().await;
    surplus_posts.clone()
}

// Function to assign a driver to a surplus food post
#[update]
async fn assign_driver_to_surplus_post(driver_id: u32, surplus_post_id: u32) -> Result<(), String> {
    // Check if the driver profile exists
    let drivers = DRIVERS.read().await;
    let driver_index = drivers.iter().position(|d| d.id == driver_id);
    if driver_index.is_none() {
        return Err(String::from("Driver profile does not exist"));
    }
    
    // Check if the surplus post exists and is not already assigned
    let surplus_posts = SURPLUS_POSTS.read().await;
    let surplus_post_index = surplus_posts.iter().position(|s| s.id == surplus_post_id && !s.assigned);
    if surplus_post_index.is_none() {
        return Err(String::from("Surplus post does not exist or is already assigned"));
    }
    
    // Update the assigned driver and assigned status of the surplus post
    let mut surplus_post = surplus_posts[surplus_post_index.unwrap()].clone();
    surplus_post.assigned = true;
    surplus_post.assigned_to = Some(driver_id);
    SURPLUS_POSTS.write().await[surplus_post_index.unwrap()] = surplus_post;
    
    // Generate a unique ID for the assignment
    let id = ID_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        *counter += 1;
        *counter
    });
    
    // Create a new assignment and add it to the ASSIGNMENTS global variable
    let assignment = Assignment { id, driver_id, surplus_post_id, assigned_at: ic_cdk::api::time() };
    ASSIGNMENTS.write().await.push(assignment);
    
    Ok(())
}

// Function to get all the assignments of drivers to surplus food posts
#[query]
async fn get_assignments() -> Vec<Assignment> {
    let assignments = ASSIGNMENTS.read().await;
    assignments.clone()
}

// Function to record the delivery of a surplus food post by a driver
#[update]
async fn record_surplus_delivery(driver_id: u32, surplus_post_id: u32, rating: u32) -> Result<(), String> {
    // Check if the driver profile exists
    let drivers = DRIVERS.read().await;
    let driver_index = drivers.iter().position(|d| d.id == driver_id);
    if driver_index.is_none() {
        return Err(String::from("Driver profile does not exist"));
    }
    
    // Check if the surplus post exists and is assigned to the driver
    let surplus_posts = SURPLUS_POSTS.read().await;
    let surplus_post_index = surplus_posts.iter().position(|s| s.id == surplus_post_id && s.assigned && s.assigned_to == Some(driver_id));
    if surplus_post_index.is_none() {
        return Err(String::from("Surplus post does not exist or is not assigned to the driver"));
    }
    
    // Update the delivered status of the surplus post
    let mut surplus_post = surplus_posts[surplus_post_index.unwrap()].clone();
    surplus_post.assigned = false;
    surplus_post.assigned_to = None;
    SURPLUS_POSTS.write().await[surplus_post_index.unwrap()] = surplus_post;
    
    // Generate a unique ID for the surplus record
    let id = ID_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        *counter += 1;
        *counter
    });
    
    // Create a new surplus record and add it to the SURPLUS_RECORDS global variable
    let surplus_record = SurplusRecord { id, surplus_post_id, driver_id, delivered_at: ic_cdk::api::time(), rating };
    SURPLUS_RECORDS.write().await.push(surplus_record);
    
    Ok(())
}

// Function to get all the records of delivered surplus posts by drivers
#[query]
async fn get_surplus_records() -> Vec<SurplusRecord> {
    let surplus_records = SURPLUS_RECORDS.read().await;
    surplus_records.clone()
}

//=================================================================================================//=================================================================================================
// Food request functions
//=================================================================================================

// Function to add a new food request
#[update]
async fn add_food_request(receiver_id: u32, description: String, quantity: u32) -> Result<(), String> {
    // Check if the receiver profile exists
    let receivers = RECEIVERS.read().await;
    let receiver_index = receivers.iter().position(|r| r.id == receiver_id);
    if receiver_index.is_none() {
        return Err(String::from("Receiver profile does not exist"));
    }
    
    // Generate a unique ID for the food request
    let id = ID_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        *counter += 1;
        *counter
    });
    
    // Create a new food request and add it to the FOOD_REQUESTS global variable
    let food_request = FoodRequest { id, receiver_id, description, quantity, assigned: false, assigned_to: None };
    FOOD_REQUESTS.write().await.push(food_request);
    
    Ok(())
}

// Function to get all the food requests
#[query]
async fn get_food_requests() -> Vec<FoodRequest> {
    let food_requests = FOOD_REQUESTS.read().await;
    food_requests.clone()
}

// Function to assign a driver to a food request
#[update]
async fn assign_driver_to_food_request(driver_id: u32, food_request_id: u32) -> Result<(), String> {
    // Check if the driver profile exists
    let drivers = DRIVERS.read().await;
    let driver_index = drivers.iter().position(|d| d.id == driver_id);
    if driver_index.is_none() {
        return Err(String::from("Driver profile does not exist"));
    }
    
    // Check if the food request exists and is not already assigned
    let food_requests = FOOD_REQUESTS.read().await;
    let food_request_index = food_requests.iter().position(|r| r.id == food_request_id && !r.assigned);
    if food_request_index.is_none() {
        return Err(String::from("Food request does not exist or is already assigned"));
    }
    
    // Update the assigned driver and assigned status of the food request
    let mut food_request = food_requests[food_request_index.unwrap()].clone();
    food_request.assigned = true;
    food_request.assigned_to = Some(driver_id);
    FOOD_REQUESTS.write().await[food_request_index.unwrap()] = food_request;
    
    // Generate a unique ID for the assignment
    let id = ID_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        *counter += 1;
        *counter
    });
    
    // Create a new assignment and add it to the ASSIGNMENTS global variable
    let assignment = Assignment { id, driver_id, food_request_id, assigned_at: ic_cdk::api::time() };
    ASSIGNMENTS.write().await.push(assignment);
    
    Ok(())
}

// Function to get all the assignments of drivers to food requests
#[query]
async fn get_food_request_assignments() -> Vec<Assignment> {
    let assignments = ASSIGNMENTS.read().await;
    assignments.clone()
}

// Function to record the delivery of a food request by a driver
#[update]
async fn record_food_delivery(driver_id: u32, food_request_id: u32, rating: u32) -> Result<(), String> {
    // Check if the driver profile exists
    let drivers = DRIVERS.read().await;
    let driver_index = drivers.iter().position(|d| d.id == driver_id);
    if driver_index.is_none() {
        return Err(String::from("Driver profile does not exist"));
    }
    
    // Check if the food request exists and is assigned to the driver
    let food_requests = FOOD_REQUESTS.read().await;
    let food_request_index = food_requests.iter().position(|r| r.id == food_request_id && r.assigned && r.assigned_to == Some(driver_id));
    if food_request_index.is_none() {
        return Err(String::from("Food request does not exist or is not assigned to the driver"));
    }
    
    // Update the delivered status of the food request
    let mut food_request = food_requests[food_request_index.unwrap()].clone();
    food_request.assigned = false;
    food_request.assigned_to = None;
    FOOD_REQUESTS.write().await[food_request_index.unwrap()] = food_request;
    
    // Generate a unique ID for the food record
    let id = ID_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        *counter += 1;
        *counter
    });
    
    // Create a new food record and add it to the FOOD_RECORDS global variable
    let food_record = FoodRecord { id, food_request_id, driver_id, delivered_at: ic_cdk::api::time(), rating };
    FOOD_RECORDS.write().await.push(food_record);
    
    Ok(())
}

// Function to get all the records of delivered food requests by drivers
#[query]
async fn get_food_records() -> Vec<FoodRecord> {
    let food_records = FOOD_RECORDS.read().await;
    food_records.clone()
}//=================================================================================================
// Messaging functions
//=================================================================================================

// Function to send a message from one user to another
#[update]
async fn send_message(sender_id: u32, recipient_id: u32, message_content: String) -> Result<(), String> {
    // Check if both sender and recipient profiles exist
    let receivers = RECEIVERS.read().await;
    let drivers = DRIVERS.read().await;
    let sender_receiver_index = receivers.iter().position(|r| r.id == sender_id);
    let sender_driver_index = drivers.iter().position(|d| d.id == sender_id);
    let recipient_receiver_index = receivers.iter().position(|r| r.id == recipient_id);
    let recipient_driver_index = drivers.iter().position(|d| d.id == recipient_id);
    
    if sender_receiver_index.is_none() && sender_driver_index.is_none() {
        return Err(String::from("Sender profile does not exist"));
    }
    if recipient_receiver_index.is_none() && recipient_driver_index.is_none() {
        return Err(String::from("Recipient profile does not exist"));
    }
    
    // Generate a unique ID for the message
    let id = ID_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        *counter += 1;
        *counter
    });
    
    // Create a new message and add it to the MESSAGES global variable
    let message = Message { id, sender_id, recipient_id, message_content, sent_at: ic_cdk::api::time() };
    MESSAGES.write().await.push(message);
    
    Ok(())
}

// Function to get all the messages of a user
#[query]
async fn get_messages(user_id: u32) -> Vec<Message> {
    let messages = MESSAGES.read().await;
    let user_messages = messages.iter().filter(|m| m.sender_id == user_id || m.recipient_id == user_id).cloned().collect();
    user_messages
}

// Function to delete a message with a specific ID
#[update]
async fn delete_message(message_id: u32) -> Result<(), String> {
    let messages = MESSAGES.read().await;
    let message_index = messages.iter().position(|m| m.id == message_id);
    
    if message_index.is_none() {
        return Err(String::from("Message does not exist"));
    }
    
    MESSAGES.write().await.remove(message_index.unwrap());
    
    Ok(())
}

//=================================================================================================
// Tests
//=================================================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_receiver() {
        ic_cdk::setup();
        
        let response = add_receiver(String::from("John Doe"), String::from("johndoe@example.com"), String::from("555-1234"));
        assert!(response.is_ok());
        
        ic_cdk::api::call::assert_num_wasm_executed_instructions(500);
    }
    
    #[test] 
    fn test_add_driver() {
        ic_cdk::setup();
        
        let response = add_driver(String::from("Jane Doe"), String::from("janedoe@example.com"), String::from("555-5678"), String::from("ABC123"));
        assert!(response.is_ok());
        
        ic_cdk::api::call::assert_num_wasm_executed_instructions(500);
    }
    
    #[test]
    fn test_add_food_request() {
        ic_cdk::setup();
        
        let response = add_food_request(1, String::from("Pizza"), 2);
        assert!(response.is_ok());
        
        ic_cdk::api::call::assert_num_wasm_executed_instructions(500);
    }
    
    #[test]
    fn test_assign_driver_to_food_request() {
        ic_cdk::setup();
        
        // Add a receiver
        add_receiver(String::from("John Doe"), String::from("johndoe@example.com"), String::from("555-1234")).unwrap();
        
        // Add a driver
        add_driver(String::from("Jane Doe"), String::from("janedoe@example.com"), String::from("555-5678"), String::from("ABC123")).unwrap();
        
        // Add a food request
        add_food_request(1, String::from("Pizza"), 2).unwrap();
        
        // Assign the driver to the food request
        let response = assign_driver_to_food_request(1, 1);
        assert!(response.is_ok());
        
        ic_cdk::api::call::assert_num_wasm_executed_instructions(500);
    }
    
    #[test]
    fn test_record_food_delivery() {
        ic_cdk::setup();
        
        // Add a receiver
        add_receiver(String::from("John Doe"), String::from("johndoe@example.com"), String::from("555-1234")).unwrap();
        
        // Add a driver
        add_driver(String::from("Jane Doe"), String::from("janedoe@example.com"), String::from("555-5678"), String::from("ABC123")).unwrap();
        
        // Add a food request
        add_food_request(1, String::from("Pizza"), 2).unwrap();
        
        // Assign the driver to the food request
        assign_driver_to_food_request(1, 1).unwrap();
        
        // Record the delivery of the food request by the driver
        let response = record_food_delivery(1, 1, 5);
        assert!(response.is_ok());
        
        ic_cdk::api::call::assert_num_wasm_executed_instructions(500);
    }
    
    #[test]
    fn test_send_message() {
        ic_cdk::setup();
        
        let response = send_message(1, 2, String::from("Hello"));
        assert!(response.is_ok());
        
        ic_cdk::api::call::assert_num_wasm_executed_instructions(500);
    }
    
    #[test]
    fn test_delete_message() {
        ic_cdk::setup();
        
        // Send a message
        send_message(1, 2, String::from("Hello")).unwrap();
        
        // Delete the message
        let response = delete_message(1);
        assert!(response.is_ok());
        
        ic_cdk::api::call::assert_num_wasm_executed_instructions(500);
    }
}
