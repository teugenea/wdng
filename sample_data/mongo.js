let user = 'wdng';
let pass = 'pass';

if (db.getUser(user) == null) {
    db.createUser(
        {
            user: user,
            pwd: pass,
            roles: ["readWrite"]
        }
    );
}

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
