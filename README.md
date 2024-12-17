# About
A demo is available at https://gojoe.dev/search

I created this search engine so that I can perform search queries across my entire ebook library. Some time ago I became interested in the idea of reading books in a non-linear manner.
Think instead of reading a book from start to finish, reading it by starting in the middle, or at the end, or hopping around different pages forwards and backwards.
The idea of hopping across all my books followed naturally. Then I thought "What if I could hop around acorrding to some theme?"
The search engine was born of this idea. For example I can type in "Plato" and get a list of passages across all my ebooks where Plato is mentioned. 

## Functionality
- Implementation of Term Frequency-Inverse Document Frequency to determine how relevant each word is to each book and across all books.
- Implementation of Cosine Similarity to rank the relevance of each book to a given search query.
- For each book that is deemed relevant a collection of snippets are generated which allows the user to see the context in which the query words appear in the book.
- Foreign function interface which allows a MacOS app I created to directly call the Rust search engine functions.
- Search index is stored in an SQLite database so that the performance cost of reading an ebook file is paid only once
