//! Framework-style core API (middle/higher-level responsibilities)
//!
//! This crate provides the runtime building blocks for applications using the
//! framework. It focuses on the orchestration, lifecycle, and common services
//! (time, scheduling, resources, events, and service registry). Rendering and
//! platform integration are provided separately via renderer-api / platform
//! crates.

// ----------------------------- Time / Clock -----------------------------

pub mod time {
    /// A very small clock abstraction used by the framework.
    #[derive(Debug, Clone, Copy)]
    pub struct Clock {
        pub ticks: u64,
        pub delta_seconds: f32,
        pub elapsed_seconds: f32,
    }

    impl Clock {
        pub fn new() -> Self {
            Self {
                ticks: 0,
                delta_seconds: 0.0,
                elapsed_seconds: 0.0,
            }
        }

        /// Advance the clock by a delta (seconds).
        pub fn advance(&mut self, delta_seconds: f32) {
            self.ticks = self.ticks.wrapping_add(1);
            self.delta_seconds = delta_seconds;
            self.elapsed_seconds += delta_seconds;
        }
    }
}

// ----------------------------- Systems / Scheduler -----------------------------

pub mod systems {
    use crate::time::Clock;

    /// System: a unit of work that runs each frame (or on a schedule).
    pub trait System: Send {
        fn update(&mut self, clock: &Clock);
    }

    /// Simple deterministic scheduler: runs systems in registration order.
    pub struct Scheduler {
        systems: Vec<Box<dyn System>>,
    }

    impl Scheduler {
        pub fn new() -> Self {
            Self { systems: Vec::new() }
        }

        pub fn add_system(&mut self, s: Box<dyn System>) {
            self.systems.push(s);
        }

        pub fn update(&mut self, clock: &Clock) {
            for sys in &mut self.systems {
                sys.update(clock);
            }
        }
    }
}

// ----------------------------- Services / Registry -----------------------------

pub mod services {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;

    /// A tiny type-indexed service registry. Services are stored by TypeId
    /// and retrieved by type.
    pub struct ServiceRegistry {
        map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    }

    impl ServiceRegistry {
        pub fn new() -> Self {
            Self { map: HashMap::new() }
        }

        pub fn insert<T: Any + Send + Sync>(&mut self, service: T) {
            self.map.insert(TypeId::of::<T>(), Box::new(service));
        }

        pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
            self.map.get(&TypeId::of::<T>())
                .and_then(|b| b.downcast_ref::<T>())
        }

        pub fn get_mut<T: Any + Send + Sync>(&mut self) -> Option<&mut T> {
            self.map.get_mut(&TypeId::of::<T>())
                .and_then(|b| b.downcast_mut::<T>())
        }
    }
}

// ----------------------------- Resources / Assets -----------------------------

pub mod resources {
    use std::collections::HashMap;

    /// Minimal asset registry concept: map string keys to opaque handles.
    pub struct AssetRegistry {
        inner: HashMap<String, String>, // placeholder: key -> path/metadata
    }

    impl AssetRegistry {
        pub fn new() -> Self {
            Self { inner: HashMap::new() }
        }

        pub fn register(&mut self, key: impl Into<String>, info: impl Into<String>) {
            self.inner.insert(key.into(), info.into());
        }

        pub fn lookup(&self, key: &str) -> Option<&String> {
            self.inner.get(key)
        }
    }
}

// ----------------------------- Events / Messaging -----------------------------

pub mod events {
    use std::collections::HashMap;

    /// Extremely small event bus keyed by string topics. Subscribers take a
    /// &str payload for simplicity.
    pub type Subscriber = Box<dyn Fn(&str) + Send + Sync>;

    pub struct EventBus {
        subscribers: HashMap<String, Vec<Subscriber>>,
    }

    impl EventBus {
        pub fn new() -> Self {
            Self { subscribers: HashMap::new() }
        }

        pub fn subscribe(&mut self, topic: impl Into<String>, f: Subscriber) {
            self.subscribers
                .entry(topic.into())
                .or_default()
                .push(f);
        }

        pub fn publish(&self, topic: &str, payload: &str) {
            if let Some(list) = self.subscribers.get(topic) {
                for s in list {
                    s(payload);
                }
            }
        }
    }
}

// ----------------------------- Lifecycle / Runtime -----------------------------

pub mod lifecycle {
    /// Application lifecycle states.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AppState {
        Starting,
        Running,
        Paused,
        Stopping,
        Stopped,
    }
}

// ----------------------------- Runtime (no backwards compatibility) -----------------------------

use time::Clock;
use systems::Scheduler;
use services::ServiceRegistry;
use resources::AssetRegistry;
use events::EventBus;
use lifecycle::AppState;

/// Framework runtime. The runtime owns scheduling, services, resources and
/// events. Platform integrations can be added without coupling the core to a
/// renderer, audio backend, or application-specific crate.
pub struct Runtime {
    clock: Clock,
    scheduler: Scheduler,
    services: ServiceRegistry,
    assets: AssetRegistry,
    events: EventBus,
    state: AppState,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            clock: Clock::new(),
            scheduler: Scheduler::new(),
            services: ServiceRegistry::new(),
            assets: AssetRegistry::new(),
            events: EventBus::new(),
            state: AppState::Starting,
        }
    }

    /// Advance the runtime and run systems once.
    pub fn update(&mut self, delta_seconds: f32) {
        self.clock.advance(delta_seconds);

        if self.state == AppState::Starting {
            self.state = AppState::Running;
        }

        self.scheduler.update(&self.clock);
        self.events.publish("frame/tick", &format!("{}", self.clock.ticks));
    }

    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    // Subsystem accessors
    pub fn scheduler_mut(&mut self) -> &mut Scheduler {
        &mut self.scheduler
    }

    pub fn services_mut(&mut self) -> &mut ServiceRegistry {
        &mut self.services
    }

    pub fn assets_mut(&mut self) -> &mut AssetRegistry {
        &mut self.assets
    }

    pub fn events_mut(&mut self) -> &mut EventBus {
        &mut self.events
    }

    pub fn state(&self) -> AppState {
        self.state
    }
}

/// Application hooks for the winit event loop. Implementors can use any Rust
/// crate for their application state and add optional framework services.
pub trait Application: 'static {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        _event: winit::event::WindowEvent,
    ) {
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}
}

struct EventLoopApplication<A> {
    application: A,
}

impl<A: Application> winit::application::ApplicationHandler for EventLoopApplication<A> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.application.resumed(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        self.application.window_event(event_loop, window_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.application.about_to_wait(event_loop);
    }
}

/// Run an application on winit's platform event loop.
pub fn run<A: Application>(application: A) -> Result<(), winit::error::EventLoopError> {
    let event_loop = winit::event_loop::EventLoop::new()?;
    let mut application = EventLoopApplication { application };
    event_loop.run_app(&mut application)
}
