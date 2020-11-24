
enum Node<K, T> {
    Inner(InnerNode<K, T>),
    Leaf(LeafNode<K, T>)
}

struct InnerNode<K, T> {
    keys: Vec<K>,
    children: Vec<Node<K, T>> 
}

//Maybe think about this at a later momment
type LeafLink<K, T> = Option<&LeafNode<K, T>>;

struct LeafNode<K, T> {
    kvs: Vec<KeyValuePair>,
    next: LeafLink<K, T>, // Make all this ones None for now?
}

struct KeyValuePair<K, T> {
    key: K,
    value: T,
}

struct LazyTree<K, T> {

}