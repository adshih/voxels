use glam::Vec3;

pub enum WorldEvent {
    PlayerMoved { id: u32, pos: Vec3, look: Vec3 },
}
