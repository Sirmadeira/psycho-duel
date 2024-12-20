use crate::client::egui::ChangeCharEvent;
use crate::client::egui::Parts;
use crate::shared::ClientId;
use bevy::prelude::*;
use bevy::utils::HashMap;
use leafwing_input_manager::prelude::*;
use lightyear::client::components::LerpFn;
use lightyear::prelude::client::ComponentSyncMode;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Component that tell me exactly what items that player has available to him we can easily query it via his UUid
/// Is worth noting this guy is just a wrapped hashmap with renamed functions but who know perhaps later he will have more to it
#[derive(Component, Reflect, Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
pub struct Inventory {
    pub items: HashMap<Uuid, Item>,
}
impl Inventory {
    /// Creates an empty inventory
    pub fn empty() -> Self {
        Self {
            items: HashMap::new(), // Assuming `items` is a `HashMap`
        }
    }
    /// Insert one sole item in our inventory
    pub fn insert_item(&mut self, item: Item) -> &Self {
        self.items.insert(item.id, item);
        self
    }
    /// Insert multiple items at once in our inventory and return self
    pub fn insert_mult_items(&mut self, items: Vec<Item>) -> &mut Self {
        for item in items {
            self.items.insert(item.id, item);
        }
        self
    }
    /// In takes a item and removes it from invetory
    pub fn remove_item(&mut self, item: &Item) -> &Self {
        self.items.remove(&item.id);
        self
    }
}

/// Item is an abstraction utilized to easy our management of what player has, when it comes to Assets
/// Things like, guns, visuals, should be items
#[derive(Reflect, Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Item {
    /// A unique identifier for our items necessary to make exclusive mechanics, if one day I want to make this display pretty
    /// Use InspectorEguiImpl
    pub id: Uuid,
    /// Name of that item - Mostly used for pretty egui
    pub name: Name,
    /// File path - File path to that grab that item via AssetCollections, must outlive it is referebce. Also serialize dislikes lifetimes
    pub file_path: String,
    /// Item type shall give me additional information like value and so on
    pub item_type: ItemType,
}

impl Item {
    /// Creates an item from it is given file path, grabing the last part from slice.
    pub fn new_from_filepath(file_path: &str) -> Self {
        let name_str = file_path
            .split("/")
            .last()
            .unwrap_or(&file_path)
            .to_string();

        let item_type = ItemType::type_from_filepath(file_path);

        Self {
            id: Uuid::new_v4(),
            name: Name::new(name_str),
            file_path: file_path.to_string(),
            item_type: item_type,
        }
    }
}
/// Display trait for item, shows us his name made for pretty :)
impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Tell me the exact type of that item, perhaps later we can make unique keys for each one
#[derive(Debug, Serialize, Deserialize, Reflect, PartialEq, Eq, Clone)]
pub enum ItemType {
    Visual,
    Weapon,
    Skeleton,
}

impl ItemType {
    /// That item type value
    pub fn value(&self) -> f32 {
        match self {
            ItemType::Visual => 1.0,
            ItemType::Skeleton => 100.0,
            ItemType::Weapon => 10.0,
        }
    }
    /// Return the item type according to the given father folder
    pub fn type_from_filepath(file_path: &str) -> Self {
        if file_path.contains("visual_parts") {
            ItemType::Visual
        } else if file_path.contains("weapon") {
            ItemType::Weapon
        } else if file_path.contains("anim_skeletons") {
            ItemType::Skeleton
        } else {
            // SPOOKY SPOOKY SKELETONS running down your spine
            ItemType::Skeleton
        }
    }
}

/// Component responsible to tell me how much money a specific client id has
/// It has mathematical function to ease our usage
#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Reflect)]
pub struct Currency {
    pub amount: f32,
}

impl Default for Currency {
    fn default() -> Self {
        Self { amount: 1.0 }
    }
}

impl Currency {
    pub fn add(&mut self, value: f32) {
        self.amount += value;
    }
    pub fn sub(&mut self, value: f32) {
        self.amount -= value;
    }
}

/// Essential struct that marks our player predicted entity.
#[derive(Component, Reflect, Serialize, Deserialize, PartialEq, Clone)]
pub struct PlayerMarker;

/// Points out who is the client id of the given player entity.
/// IMPORTANT- Every entity that is reasonably used, should have similar identifiers
#[derive(Component, Reflect, Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PlayerId {
    /// Base client id of this player
    pub id: ClientId,
}

/// Essential component utilized to tell me what exactly are the scenes player should have as children
/// Important- Why servertoclient? Well we wanna validate if everything occured succesfully, if so we are going to change in server via messages
/// Also to mantain a monolithic player entity
#[derive(Component, Reflect, Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PlayerVisuals {
    /// Base skeleton to fit into
    pub skeleton: Item,
    /// Character available head
    pub head: Item,
    /// Character available torso
    pub torso: Item,
    /// Character available legs
    pub leg: Item,
    /// Character available weapon
    pub weapon_1: Item,
}

impl Default for PlayerVisuals {
    fn default() -> Self {
        Self {
            head: Item::new_from_filepath("characters/visual_parts/suit_head.glb"),
            torso: Item::new_from_filepath("characters/visual_parts/scifi_torso.glb"),
            leg: Item::new_from_filepath("characters/visual_parts/witch_legs.glb"),
            weapon_1: Item::new_from_filepath("weapons/katana.glb"),
            skeleton: Item::new_from_filepath("characters/anim_skeletons/main_skeleton.glb"),
        }
    }
}

impl PlayerVisuals {
    /// Returns an iterator over the visual components. Good iterator for when spawning first the entity
    pub fn iter_visuals(&self) -> impl Iterator<Item = &Item> {
        vec![&self.head, &self.torso, &self.leg, &self.skeleton].into_iter()
    }
    /// Returns a reference to the visual component corresponding to the given`Parts` enum
    /// Avoids the usage of uncessary match statements
    pub fn get_visual(&self, part: &Parts) -> &Item {
        match part {
            Parts::Head => &self.head,
            Parts::Torso => &self.torso,
            Parts::Leg => &self.leg,
        }
    }
    /// Returns a mutable reference to the visual component corresponding to the given`Parts` enum
    /// Avoids the usage of uncessary match statements
    pub fn get_visual_mut(&mut self, part: &Parts) -> &mut Item {
        match part {
            Parts::Head => &mut self.head,
            Parts::Torso => &mut self.torso,
            Parts::Leg => &mut self.leg,
        }
    }
}
/// Abstraction - Utilized to tell me what is the current action of player, gonna be essential for animations and movement
/// Unfortunately he is not inserted via protocol, although we could treat him as a component. Why? Because of reasons!
/// I think it has something to do with action state which is the actual interesting component.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Reflect, Clone, Copy, Hash)]
pub enum PlayerActions {
    /// Forward direction
    Forward,
    /// Backward direction
    Backward,
    /// Left direction
    Left,
    /// Goes in the cam.right direction
    Right,
}

impl Actionlike for PlayerActions {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            Self::Forward => InputControlKind::Button,
            Self::Backward => InputControlKind::Button,
            Self::Left => InputControlKind::Button,
            Self::Right => InputControlKind::Button,
        }
    }
}

impl PlayerActions {
    /// Return the default input map for that player actions. A usefull way of aligning both client and server with the same default input map
    pub fn default_input_map() -> InputMap<Self> {
        let input_map = InputMap::default()
            .with(Self::Forward, KeyCode::KeyW)
            .with(Self::Forward, KeyCode::ArrowUp)
            .with(Self::Backward, KeyCode::KeyS)
            .with(Self::Backward, KeyCode::ArrowDown)
            .with(Self::Left, KeyCode::KeyA)
            .with(Self::Left, KeyCode::ArrowLeft)
            .with(Self::Right, KeyCode::ArrowDown);
        return input_map;
    }
}

/// Our save resource map, it is gonna store all types of core information really important for our mechanics.
/// Initially this is monolithic, meaning we only have one save info map that stores basically all of user info later we can make subdivisions and subfiles
#[derive(Resource, Serialize, Deserialize, Clone, Debug, PartialEq, Reflect, Default)]
#[reflect(Resource)]
pub struct CoreSaveInfoMap {
    pub map: HashMap<ClientId, CoreInformation>,
}

/// A centralization struct - That shall store everything that NEEDs to be saved about that specific client
/// It also stores a client_id pointer
#[derive(Bundle, Serialize, Deserialize, Reflect, Clone, Debug, PartialEq)]
pub struct CoreInformation {
    pub player_id: PlayerId,
    pub player_visuals: PlayerVisuals,
    pub inventory: Inventory,
    pub currency: Currency,
}

impl CoreInformation {
    /// Pass a client id - Get the default core for new players
    pub fn new(client_id: ClientId) -> Self {
        // All default player visuals
        let player_visual_items = PlayerVisuals::default();
        // Clone here to avoid utilizing same memory space as visuals
        let default_items: Vec<Item> = player_visual_items.iter_visuals().cloned().collect();
        // Fill empty inventory with default items
        let mut empty_inventory = Inventory::empty();
        empty_inventory.insert_mult_items(default_items);

        Self {
            player_id: PlayerId { id: client_id },
            player_visuals: PlayerVisuals::default(),
            currency: Currency::default(),
            inventory: empty_inventory,
        }
    }
}

/// A bidirectional message utilized, to save things on server.
/// If one of the optional fields are passed we should validate the information.
#[derive(Event, Serialize, Deserialize, Clone, PartialEq)]
pub struct SaveMessage {
    pub id: ClientId,
    /// Should be filled if there was an action in client that changed that character visual
    pub change_char: Option<ChangeCharEvent>,
    /// Should be filled if there was an action that changed it is currency
    pub change_currency: Option<Currency>,
    /// Should occur whenever our inventory changes via buy or sell actions
    pub change_inventory: Option<Inventory>,
}

/// Centralization plugin - Defines how our component will be synced (from server to client or client to server or bidirectional)
/// Defines what essential components need to be replicated among the two.
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // Leafwing input plugin from lightyear - Responsible for handling all the rollback and go back mechanics
        // Worth mentioning- Only occurs when all plugins added
        // Warning - Does not work with leafwing resources
        app.add_plugins(LeafwingInputPlugin::<PlayerActions>::default());
        // -> First - Spawn an entity with replicate component on server, after you do that this api applies it is logic
        // -> Second - Register component, means that component will be available on the replicated entity on client
        // -> Third - ChannelDirection tells me if it is server to client or client  to server, the replication direction.
        // Most cases is the first method (Server authoritative avoids hacks)
        // -> Fourth - Add prediction, inserts that component on the predicted entity.
        // -> Fifth - ComponentSyncMode tell me, how many time we should send that information from confirmed entity, to predicted.
        app.register_component::<PlayerMarker>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once);
        app.register_component::<PlayerId>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once);
        app.register_component::<PlayerVisuals>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Simple);
        app.register_component::<Inventory>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Simple);
        app.register_component::<Currency>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Simple);
        app.register_component::<Name>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Once)
            .add_interpolation(ComponentSyncMode::Simple);

        // Okay this guys gets it is own comment because if we fuck him up we screwed
        app.register_component::<Transform>(ChannelDirection::ServerToClient)
            .add_prediction(ComponentSyncMode::Full)
            .add_correction_fn(TransformLinearInterpolation::lerp);

        // Replicated resources -  The workflow for replicated resources is as follows
        //-> First - Register in shared, as he is supposed to exist both in client and server
        //-> Second - Initialize him in server and client
        //-> Third - Do commands.replicate to start replicating him when necessary
        app.register_resource::<CoreSaveInfoMap>(ChannelDirection::ServerToClient);

        // Self-made messages - The workflow for messages is as follows:
        // -> First register message
        // -> Send her via clientconnectionmessager using send_message function with all of it is shenanigans
        // -> Read it via EventReader<MessageEvent<>>
        app.register_message::<SaveMessage>(ChannelDirection::Bidirectional);

        // Debug registering
        app.register_type::<PlayerId>();
        app.register_type::<PlayerVisuals>();
        app.register_type::<CoreSaveInfoMap>();
    }
}

pub struct TransformLinearInterpolation;

impl LerpFn<Transform> for TransformLinearInterpolation {
    fn lerp(start: &Transform, other: &Transform, t: f32) -> Transform {
        let translation = start.translation * (1.0 - t) + other.translation * t;
        let rotation = start.rotation.slerp(other.rotation, t);
        let scale = start.scale * (1.0 - t) + other.scale * t;
        let res = Transform {
            translation,
            rotation,
            scale,
        };
        trace!(
            "position lerp: start: {:?} end: {:?} t: {} res: {:?}",
            start,
            other,
            t,
            res
        );
        res
    }
}
