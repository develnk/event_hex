/*
*  Identity and Access Management Context (IAM). Responsible for authentication, authorization, and binding external
*  identifiers to internal entities. This is a critically important auxiliary layer that protects the system domain.
*  IAM Context: Contains logins, password hashes, tokens and roles. Its job is to answer "Who is this?" and "What are they allowed to do?"
*/
pub mod identity;
pub mod service;
