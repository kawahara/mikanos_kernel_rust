pub mod once_cell {
    use conquer_once::noblock;
    use conquer_once::{TryGetError, TryInitError};

    #[derive(Debug)]
    pub struct OnceCell<T>(noblock::OnceCell<T>);

    impl<T> OnceCell<T> {
        pub const fn uninit() -> Self {
            Self(noblock::OnceCell::uninit())
        }

        pub fn try_init_once(&self, f: impl FnOnce() -> T) -> Result<(), TryInitError> {
            Ok(self.0.try_init_once(f)?)
        }

        pub fn init_once(&self, f: impl FnOnce() -> T) {
            self.try_init_once(f).unwrap()
        }

        pub fn try_get(&self) -> Result<&T, TryGetError> {
            Ok(self.0.try_get()?)
        }

        pub fn get(&self) -> &T {
            self.try_get().unwrap()
        }
    }
}
