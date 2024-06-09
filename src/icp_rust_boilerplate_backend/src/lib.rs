#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

// BusinessType is a custom enum type that is used to represent the type of business
#[derive(
    candid::CandidType, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default, Debug,
)]

enum BusinessType {
    #[default]
    Restaurant,
    Grocery,
    Bakery,
    Other,
}

// FoodType is a custom enum type that is used to represent the type of food
#[derive(
    candid::CandidType, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default, Debug,
)]

enum FoodType {
    #[default]
    Vegetables,
    Fruits,
    Dairy,
    Meat,
    Grains,
    Bakery,
    Beverages,
    Other,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct SurplusPost {
    id: u64,
    donor_id: String,
    food_type: FoodType,
    quantity_kg: u32,
    best_before_date: String,
    handling_instructions: String,
    assigned: bool,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct DonorProfile {
    id: u64,
    name: String,
    phone_number: String,
    email: String,
    address: String,
    business_type: BusinessType,
    created_at: u64,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct ReceiverProfile {
    id: u64,
    name: String,
    phone_number: String,
    email: String,
    address: String,
    created_at: u64,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct DriverProfile {
    id: u64,
    name: String,
    phone_number: String,
    email: String,
    address: String,
    created_at: u64,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Assignment {
    id: u64,
    receiver_id: u64,
    surplus_post_id: u64,
    driver_id: u64,
    status: String,
    created_at: u64,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct SurplusRecord {
    id: u64,
    surplus_post_id: u64,
    driver_id: u64,
    delivered_at: u64,
    rating: Option<u8>,
}

impl Storable for SurplusPost {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for SurplusPost {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for DonorProfile {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for DonorProfile {
    const MAX_SIZE: u32 = 512;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for ReceiverProfile {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for ReceiverProfile {
    const MAX_SIZE: u32 = 512;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for DriverProfile {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for DriverProfile {
    const MAX_SIZE: u32 = 512;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for Assignment {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Assignment {
    const MAX_SIZE: u32 = 512;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for SurplusRecord {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for SurplusRecord {
    const MAX_SIZE: u32 = 512;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static DONORS_STORAGE: RefCell<StableBTreeMap<u64, DonorProfile, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static RECEIVERS_STORAGE: RefCell<StableBTreeMap<u64, ReceiverProfile, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    static DRIVERS_STORAGE: RefCell<StableBTreeMap<u64, DriverProfile, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));

    static SURPLUS_POSTS_STORAGE: RefCell<StableBTreeMap<u64, SurplusPost, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
    ));

    static ASSIGNMENTS_STORAGE: RefCell<StableBTreeMap<u64, Assignment, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5)))
    ));

    static SURPLUS_RECORDS_STORAGE: RefCell<StableBTreeMap<u64, SurplusRecord, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6)))
    ));
}

// Donor Payload
#[derive(candid::CandidType, Deserialize, Serialize)]
struct DonorPayload {
    name: String,
    phone_number: String,
    email: String,
    address: String,
    business_type: BusinessType,
}

// Receiver Payload
#[derive(candid::CandidType, Deserialize, Serialize)]
struct ReceiverPayload {
    name: String,
    phone_number: String,
    email: String,
    address: String,
}

// Driver Payload
#[derive(candid::CandidType, Deserialize, Serialize)]
struct DriverPayload {
    name: String,
    phone_number: String,
    email: String,
    address: String,
}

// Surplus Post Payload
#[derive(candid::CandidType, Deserialize, Serialize)]
struct SurplusPostPayload {
    donor_id: String,
    food_type: FoodType,
    quantity_kg: u32,
    best_before_date: String,
    handling_instructions: String,
}

// Assignment Payload
#[derive(candid::CandidType, Deserialize, Serialize)]
struct AssignmentPayload {
    receiver_id: u64,
    surplus_post_id: u64,
    driver_id: u64,
}

// Surplus Record Payload
#[derive(candid::CandidType, Deserialize, Serialize)]
struct SurplusRecordPayload {
    surplus_post_id: u64,
    driver_id: u64,
}

// Function to create a new donor profile
#[ic_cdk::update]
fn create_donor_profile(payload: DonorPayload) -> Result<DonorProfile, String> {
    // Validate the payload to ensure that the required fields are present
    if payload.name.is_empty()
        && payload.phone_number.is_empty()
        && payload.email.is_empty()
        && payload.address.is_empty()
    {
        return Err("All fields are required".to_string());
    }

    // Validate the payload to ensure that the email format is correct
    if !payload.email.contains('@') {
        return Err("Invalid email format".to_string());
    }

    // Ensure email address uniqueness
    let email_exists = DONORS_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .any(|(_, donor)| donor.email == payload.email)
    });
    if email_exists {
        return Err("Email already exists".to_string());
    }

    // Validate the payload to ensure that the phone number format is correct
    if payload.phone_number.len() != 10 {
        return Err("Invalid phone number format".to_string());
    }
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let donor_profile = DonorProfile {
        id,
        name: payload.name,
        phone_number: payload.phone_number,
        email: payload.email,
        address: payload.address,
        business_type: payload.business_type,
        created_at: time(),
    };

    DONORS_STORAGE.with(|storage| storage.borrow_mut().insert(id, donor_profile.clone()));

    Ok(donor_profile)
}

// Function to create a new receiver profile
#[ic_cdk::update]
fn create_receiver_profile(payload: ReceiverPayload) -> Result<ReceiverProfile, String> {
    // Validate the payload to ensure that the required fields are present
    if payload.name.is_empty()
        && payload.phone_number.is_empty()
        && payload.email.is_empty()
        && payload.address.is_empty()
    {
        return Err("All fields are required".to_string());
    }

    // Validate the payload to ensure that the email format is correct
    if !payload.email.contains('@') {
        return Err("Invalid email format".to_string());
    }

    // Ensure email address uniqueness
    let email_exists = RECEIVERS_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .any(|(_, receiver)| receiver.email == payload.email)
    });
    if email_exists {
        return Err("Email already exists".to_string());
    }

    // Validate the payload to ensure that the phone number format is correct
    if payload.phone_number.len() != 10 {
        return Err("Invalid phone number format".to_string());
    }
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let receiver_profile = ReceiverProfile {
        id,
        name: payload.name,
        phone_number: payload.phone_number,
        email: payload.email,
        address: payload.address,
        created_at: time(),
    };

    RECEIVERS_STORAGE.with(|storage| storage.borrow_mut().insert(id, receiver_profile.clone()));

    Ok(receiver_profile)
}

// Function to create a new driver profile
#[ic_cdk::update]
fn create_driver_profile(payload: DriverPayload) -> Result<DriverProfile, String> {
    // Validate the payload to ensure that the required fields are present
    if payload.name.is_empty()
        && payload.phone_number.is_empty()
        && payload.email.is_empty()
        && payload.address.is_empty()
    {
        return Err("All fields are required".to_string());
    }

    // Validate the payload to ensure that the email format is correct
    if !payload.email.contains('@') {
        return Err("Invalid email format".to_string());
    }

    // Ensure email address uniqueness
    let email_exists = DRIVERS_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .any(|(_, driver)| driver.email == payload.email)
    });
    if email_exists {
        return Err("Email already exists".to_string());
    }

    // Validate the payload to ensure that the phone number format is correct
    if payload.phone_number.len() != 10 {
        return Err("Invalid phone number format".to_string());
    }
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let driver_profile = DriverProfile {
        id,
        name: payload.name,
        phone_number: payload.phone_number,
        email: payload.email,
        address: payload.address,
        created_at: time(),
    };

    DRIVERS_STORAGE.with(|storage| storage.borrow_mut().insert(id, driver_profile.clone()));

    Ok(driver_profile)
}

// Function to get all drivers
#[ic_cdk::query]
fn get_all_drivers() -> Result<Vec<DriverProfile>, Error> {
    DRIVERS_STORAGE.with(|storage| {
        let stable_btree_map = &*storage.borrow();

        let records: Vec<DriverProfile> = stable_btree_map
            .iter()
            .map(|(_, record)| record.clone())
            .collect();

        if records.is_empty() {
            Err(Error::NotFound {
                msg: "No drivers found.".to_string(),
            })
        } else {
            Ok(records)
        }
    })
}

// Function to create a new surplus post
#[ic_cdk::update]
fn create_surplus_post(payload: SurplusPostPayload) -> Result<SurplusPost, String> {
    // Validate the payload to ensure that the required fields are present
    if payload.donor_id.is_empty()
        && payload.quantity_kg == 0
        && payload.best_before_date.is_empty()
        && payload.handling_instructions.is_empty()
    {
        return Err("All fields are required".to_string());
    }
    
    // Validate the payload to ensure that the donor_id exists
    let donor_id: u64 = payload
        .donor_id
        .parse()
        .map_err(|_| "Invalid donor ID format".to_string())?;
    let donor_exists = DONORS_STORAGE.with(|storage| storage.borrow().contains_key(&donor_id));

    if !donor_exists {
        return Err("Donor ID does not exist".to_string());
    }

    // // Validate the payload to ensure that the best_before_date is in the future
    // if payload.best_before_date <= time() {
    //     return Err("The best before date must be in the future".to_string());
    // }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let surplus_post = SurplusPost {
        id,
        donor_id: payload.donor_id,
        food_type: payload.food_type,
        quantity_kg: payload.quantity_kg,
        best_before_date: payload.best_before_date,
        handling_instructions: payload.handling_instructions,
        assigned: false,
    };

    SURPLUS_POSTS_STORAGE.with(|storage| storage.borrow_mut().insert(id, surplus_post.clone()));

    Ok(surplus_post)
}

// Function to get all surplus posts
#[ic_cdk::query]
fn get_all_surplus_posts() -> Result<Vec<SurplusPost>, Error> {
    SURPLUS_POSTS_STORAGE.with(|storage| {
        let stable_btree_map = &*storage.borrow();

        let records: Vec<SurplusPost> = stable_btree_map
            .iter()
            .map(|(_, record)| record.clone())
            .collect();

        if records.is_empty() {
            Err(Error::NotFound {
                msg: "No surplus posts found.".to_string(),
            })
        } else {
            Ok(records)
        }
    })
}

// Function to get surplus post by food type
#[ic_cdk::query]
fn get_surplus_post_by_food_type(food_type: FoodType) -> Result<Vec<SurplusPost>, Error> {
    SURPLUS_POSTS_STORAGE.with(|storage| {
        let stable_btree_map = &*storage.borrow();

        let records: Vec<SurplusPost> = stable_btree_map
            .iter()
            .filter(|(_, record)| record.food_type == food_type)
            .map(|(_, record)| record.clone())
            .collect();

        if records.is_empty() {
            Err(Error::NotFound {
                msg: "No surplus posts found.".to_string(),
            })
        } else {
            Ok(records)
        }
    })
}

#[ic_cdk::update]
fn create_assignment(payload: AssignmentPayload) -> Result<Assignment, String> {
    // Validate the payload to ensure that the required fields are present
    if payload.surplus_post_id == 0 || payload.driver_id == 0 || payload.receiver_id == 0 {
        return Err("All fields are required".to_string());
    }

    // Validate the payload to ensure that the receiver_id exists
    let receiver_exists =
        RECEIVERS_STORAGE.with(|storage| storage.borrow().contains_key(&payload.receiver_id));
    if !receiver_exists {
        return Err("Receiver ID does not exist".to_string());
    }

    // Validate the payload to ensure that the surplus_post_id exists
    let surplus_post_exists = SURPLUS_POSTS_STORAGE
        .with(|storage| storage.borrow().contains_key(&payload.surplus_post_id));
    if !surplus_post_exists {
        return Err("Surplus post ID does not exist".to_string());
    }

    // Validate the payload to ensure that the driver_id exists
    let driver_exists =
        DRIVERS_STORAGE.with(|storage| storage.borrow().contains_key(&payload.driver_id));
    if !driver_exists {
        return Err("Driver ID does not exist".to_string());
    }

    // Validate the payload to ensure that the surplus_post_id is not already assigned
    let already_assigned = ASSIGNMENTS_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .any(|(_, assignment)| assignment.surplus_post_id == payload.surplus_post_id)
    });
    if already_assigned {
        return Err("Surplus post ID is already assigned".to_string());
    }

    // Validate the payload to ensure that the driver_id is not already assigned to another post
    let driver_assigned = ASSIGNMENTS_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .any(|(_, assignment)| assignment.driver_id == payload.driver_id)
    });
    if driver_assigned {
        return Err("Driver ID is already assigned to another post".to_string());
    }

    // Increment the ID counter and create the assignment
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let assignment = Assignment {
        id,
        surplus_post_id: payload.surplus_post_id,
        receiver_id: payload.receiver_id,
        driver_id: payload.driver_id,
        status: "Pending".to_string(),
        created_at: time(),
    };

    ASSIGNMENTS_STORAGE.with(|storage| storage.borrow_mut().insert(id, assignment.clone()));

    Ok(assignment)
}

// Function to create a new surplus record
#[ic_cdk::update]
fn create_surplus_record(payload: SurplusRecordPayload) -> Result<SurplusRecord, String> {
    // Validate the payload to ensure that the required fields are present
    if payload.surplus_post_id == 0 && payload.driver_id == 0 {
        return Err("All fields are required".to_string());
    }

    // Validate the payload to ensure that the surplus_post_id exists
    let surplus_post_exists = SURPLUS_POSTS_STORAGE
        .with(|storage| storage.borrow().contains_key(&payload.surplus_post_id));
    if !surplus_post_exists {
        return Err("Surplus post ID does not exist".to_string());
    }
    
    // Validate the payload to ensure that the driver_id exists
    let driver_exists =
        DRIVERS_STORAGE.with(|storage| storage.borrow().contains_key(&payload.driver_id));
    if !driver_exists {
        return Err("Driver ID does not exist".to_string());
    }

    // Validate the payload to ensure that the surplus_post_id is already assigned to the driver id
    let already_assigned = ASSIGNMENTS_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .any(|(_, assignment)| assignment.surplus_post_id == payload.surplus_post_id && assignment.driver_id == payload.driver_id)
    });
    if !already_assigned {
        return Err("Surplus post ID is not assigned to the driver ID".to_string());
    }
    
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let surplus_record = SurplusRecord {
        id,
        surplus_post_id: payload.surplus_post_id,
        driver_id: payload.driver_id,
        delivered_at: time(),
        rating: None,
    };

    SURPLUS_RECORDS_STORAGE.with(|storage| storage.borrow_mut().insert(id, surplus_record.clone()));

    Ok(surplus_record)
}

// Error types
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    UnAuthorized { msg: String },
}

// need this to generate candid
ic_cdk::export_candid!();
