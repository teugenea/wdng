let db = connect('127.0.0.1:27017/wdng');
db.auth('wdng', 'wdngpass112');

db.lang_units.drop();
db.lang_units.insertOne(
    {
        'ru': 'уйти без наказания',
        'en': 'get away with'
    }
);

db.lang_units.createIndex({ "ru": "text", "en": "text" });
