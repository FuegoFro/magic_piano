use web_sys::{Element, HtmlCollection};

struct HtmlCollectionIterator {
    collection: HtmlCollection,
    current_index: u32,
}

impl Iterator for HtmlCollectionIterator {
    type Item = Element;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.collection.get_with_index(self.current_index);
        self.current_index += 1;
        next
    }
}

pub trait HtmlCollectionIntoIterator {
    fn into_iter(self) -> impl Iterator<Item = Element>;
}

impl HtmlCollectionIntoIterator for HtmlCollection {
    fn into_iter(self) -> impl Iterator<Item = Element> {
        HtmlCollectionIterator {
            collection: self,
            current_index: 0,
        }
    }
}
