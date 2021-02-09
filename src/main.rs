mod errors;
mod log_event;
mod stream;
mod tree;
mod compress;

use compress::xdelta;

fn main() {
//    let u = stream::read_log_events_from_file("test.json").unwrap();
//    println!("{:#?}", u);

    let s = "{
        \"hostname\": \"aws-east1\",
        \"trace_id\": \"x123n\",
        \"menssage\": \"Processing payment id: 1234asdasd13\"
      }";
    let t = "{
        \"hostname\": \"aws-east1\",
        \"trace_id\": \"z789h\",
        \"menssage\": \"Processing payment id: 45asdnc_2\"
      }";


    let z = "{
        \"hostname\": \"aws-east1\",
        \"trace_id\": \"huhaswbq2\",
        \"menssage\": \"Processing payment id: okcaojwqn22\"
      }";
    


    print!("{}", xdelta(&s, &t));
    print!("{}", xdelta(&t, &z));
}
