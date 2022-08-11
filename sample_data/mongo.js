let host = '127.0.0.1:27017/wdng';
let user = 'wdng';
let pass = 'pass';

let db = connect(host);
db.auth(user, pass);

db.lang_units.drop();
db.lang_units.insertMany([
    {
        'ru': 'уйти без наказания',
        'en': 'get away with'
    },
    {
        'en': 'implausible',
        'ru': 'неправдоподобный'
    },
    {
        'ru': 'ложный',
        'en': 'spurious'
    }
]);

db.lang_units.createIndex({ "ru": "text", "en": "text" });
