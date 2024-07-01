struct WordIterator<'s> {
    position: usize,
    string: &'s str
}

impl<'s> WordIterator<'s> {
    /// Creates a new WordIterator based on a string.
    fn new(string: &'s str) -> WordIterator {
        WordIterator {
            position: 0,
            string
        }
    }
    
    /// Gives the next word. `None` if there aren't any words left.
    fn next_word(&mut self) -> Option<&'s str> {
        let start_of_word = &self.string[self.position..];
        let index_of_next_space = start_of_word.find(' ').unwrap_or(start_of_word.len());
        if start_of_word.len() != 0 {
            self.position += index_of_next_space + 1;
            Some(&start_of_word[..index_of_next_space]) 
        } else {
            None
        }
    }
}

fn main() {
    let text = String::from("Twas brillig, and the slithy toves // Did gyre and gimble in the wabe: // All mimsy were the borogoves, // And the mome raths outgrabe. ");
    let mut word_iterator = WordIterator::new(&text);
    let mut words_vector = vec![];
    while let Some(word) = word_iterator.next_word() {
        words_vector.push(word);
    }
    println!("{words_vector:?}")
    
    // assert_eq!(word_iterator.next_word(), Some("Twas"));
    // assert_eq!(word_iterator.next_word(), Some("brillig,"));
}