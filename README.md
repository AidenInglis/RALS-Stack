# RALS-Stack
guide below

frontend:
trunk serve --open

Backend:
cargo run

Database:
sqlx migrate run 



stuff for testing in graphql

mutation {
  login(input:{email:"admin@example.com", password:"pass1234"})
}

mutation {
  login(input:{email:"aiden@aiden.aiden", password:"aiden"})
}



{ 
  "Authorization": "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJiNGEzYzg5Yi04MTY1LTQ1MGMtYjA0NS1iMjA1ZTJmMTY5ZWIiLCJleHAiOjE3NTYzMDI0OTF9.NT-PD4bKbVqdfzm6sjVi7WBOv0tOfKx0fsIYnpN-wG8"
}

mutation CreateCoupon {
  createCoupon(input:{
    code: "HELLO10",
    description: "10% off",
    service: "my-store",
    expiresInDays: 30
  }) {
    id code description service expires_at owner_id created_at
  }
}

mutation UpdateCoupon {
  updateCoupon(input:{
    code: "HELLO10",
    description: "12% off",
    expiresInDays: 45
  })
}

mutation DeleteCoupon {
  deleteCoupon(code: "HELLO10")
}

mutation {
  claim_coupon(code:"HELLO10") {
    id code owner_id expires_at
  }
}


query {
  myCoupons { code description service expires_at owner_id }
}

mutation { release_coupon(code:"HELLO10") }