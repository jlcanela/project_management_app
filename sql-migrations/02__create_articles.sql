CREATE TABLE articles (
    id SERIAL PRIMARY KEY,
    title text,
    content text,
    published_by int,
    published_on timestamp
);
