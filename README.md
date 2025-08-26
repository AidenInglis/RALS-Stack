# RALS-Stack




stuff for testing in graphql

mutation {
  login(input:{email:"admin@example.com", password:"pass1234"})
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