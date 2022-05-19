pub trait Actor {
    fn system(&self) -> ActorSystem {
        todo!();
    }
}

pub trait Tell<T>: Actor where T: Send {
    fn tell(&mut self, msg: T);
}

pub trait Ask<T>: Actor where T: Send {
    type Return: Send;

    fn ask(&mut self, msg: T) -> Self::Return;
}

struct Test;

struct Hello(String);
struct Goodbye(String);

struct Add(i32, i32);

impl Actor for Test {}

impl Tell<Hello> for Test {
    fn tell(&mut self, msg: Hello) {
        todo!()
    }
}

impl Tell<Goodbye> for Test {
    fn tell(&mut self, msg: Goodbye) {
        todo!()
    }
}

impl Ask<Add> for Test {
    type Return = i32;

    fn ask(&mut self, Add(n1, n2): Add) -> Self::Return {
         n1 + n2
    }
}

struct Handle<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> Handle<T> {
    pub fn new() -> Handle<T> {
        Handle { _marker: Default::default() }
    }

    pub fn tell<M>(&self, message: M) where T: Tell<M>, M: Send {
        todo!();
    }

    pub fn ask<M>(&self, message: M) -> T::Return where T: Ask<M>, M: Send  {
        todo!();
    }
}


pub struct ActorSystem {

}

impl ActorSystem {
    pub fn new() -> ActorSystem {
        ActorSystem {

        }
    }

    pub fn start<T>(&self, system: T) where T: Actor {
        todo!();
    }
}

fn test() {
    let system = ActorSystem::new();

    system.start(Test{});

    let h: Handle<Test> = Handle::new();

    h.tell(Goodbye("wat".into()));
    let num = h.ask(Add(1,2));
}
