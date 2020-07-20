use crate::peripherals::basic_fifo::BasicFIFO;

struct Invoke {

}

struct ComponentFifo {
    fifo: BasicFIFO
}

impl ComponentFifo {
    fn new(fifo: BasicFIFO) -> ComponentFifo {
        ComponentFifo {
            fifo
        }
    }

    fn write_invoke() {

    }
}