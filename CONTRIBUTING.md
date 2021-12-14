# Requirements to Contribute

1. Rust 1.56 or newer installed
2. Mysql C++ Driver/Connector installed
3. Mysql Database Ready for use
4. For SSL openssl library installed
5. Node 16 installed and NPM installed
6. Lots of Patience.

# Configuring nitro repo.

1. Copy example.env to your working directory of the application and name it .env
2. The only one you will need to edit will be the `DATABASE_URL` and BIND_URL if that port is already in use

# Development Frontend Only

1. Follow all steps up to this point only changing the cargo build command
   to `cargo build --release --features dev-frontend`
2. Then add a .env.local inside the site directory and add the value `VITE_API_URL=http://127.0.0.1:6742` Changing the
   URL if necessary.
3. You can at this point execute `npm run dev` with the backend executable running and work on the frontend

# Development Full Stack or Backend

1. I recommend following the Frontend development steps because using the frontend development mode will be easier
2. Ignore the --release argument. Because you are doing development. 

