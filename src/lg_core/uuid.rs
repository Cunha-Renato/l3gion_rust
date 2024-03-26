use rand::Rng;

#[derive(Default, Debug, Hash, Eq, PartialEq)]
pub struct UUID(u128);
impl UUID {
    pub fn generate() -> Self {
        let uuid = rand::thread_rng().gen::<u128>();
        Self(uuid)
    }
}
impl Clone for UUID {
    fn clone(&self) -> Self {
        Self::generate()
    } 
}