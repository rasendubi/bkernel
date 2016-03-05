//! Global variable with destructor.
//!
//! Well, that's a trick to overcome `error: statics are not allowed
//! to have destructors`.

use ::core::cell::UnsafeCell;

pub struct Global<T>(UnsafeCell<Option<T>>);
unsafe impl<T> Sync for Global<T> { }

impl<T> Global<T> {
    /// Creates a new empty `Global` variable and initializes it to
    /// `value`.
    ///
    /// # Examples
    ///
    /// ```
    /// static var: Global<u32> = Global::new(42);
    /// assert_eq!(42, var.get());
    /// ```
    #[allow(dead_code)]
    pub const fn new(value: T) -> Global<T> {
        Global(UnsafeCell::new(Some(value)))
    }

    /// Creates a new empty `Global` variable.
    /// The variable must be initialized later with `init()`.
    ///
    /// # Examples
    ///
    /// ```
    /// static var: Global<u32> = Global::new_empty();
    /// ```
    pub const fn new_empty() -> Global<T> {
        Global(UnsafeCell::new(None))
    }

    /// Initializes variable to the value.
    pub fn init(&self, value: T) {
        unsafe {
            (*self.0.get()) = Some(value);
        }
    }

    /// Gets a value as mutable reference.
    ///
    /// # Panics
    ///
    /// Panics if variable is empty.
    ///
    /// ```
    /// static var: Global<u32> = Global::init(32);
    /// var.get() += 10;
    /// assert_eq!(42, var.get());
    /// ```
    pub fn get(&self) -> &mut T {
        unsafe {
            (*self.0.get()).as_mut().unwrap() as &mut _
        }
    }
}
