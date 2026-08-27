cargo run --bin daemon -- register-service
sc start "Bored Crow"
sc failure "Bored Crow" reset= 1 actions= restart/1000