use slint::ComponentHandle;

slint::include_modules!();

pub struct WindowWrapper<W: ComponentHandle> {
    window: W,
}

impl<W: ComponentHandle> WindowWrapper<W> {
    pub fn from(window: W) -> Self { 
        Self {
            window: window
        }
    }

    pub fn get_window(&self) -> slint::Weak<W> {
        self.window.as_weak()
    }

    pub fn with_window<F>(&self, runnable: F)
    where
        F: FnOnce(&W),
    {
        runnable(&self.window);
    }
    
    pub fn get_global<'a, T>(&'a self) -> slint::Weak<<T as slint::Global<'a, W>>::StaticSelf>
    where
        T: slint::Global<'a, W>, 
    {
        self.window.global::<T>().as_weak()
    }

    pub fn with_global<'a, T, F, R>(&'a self, runnable: F) -> R
    where
        T: slint::Global<'a, W>,
        F: FnOnce(&T) -> R,
    {
        let global_instance = self.window.global::<T>();
        runnable(&global_instance)
    }

    pub fn run(&self) -> anyhow::Result<()> {
        self.window.run().map_err(Into::into)
    }
}

