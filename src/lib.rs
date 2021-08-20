use hdk::prelude::*;

// an enumerator to specify different "permissions" options
// for who can edit their Wiki Page
// eligible to be set by the author for any given WikiPage
#[derive(Serialize, Deserialize, SerializedBytes, Debug, Clone)]
pub enum UpdatePermission {
    AuthorOnly,
    Others,
}

// The shape of the main data structure
// for the application, and the "entry type definition"
// for Holochain being specified with hdk_entry "derive"
#[hdk_entry(id = "wiki_page")]
#[derive(Clone)]
pub struct WikiPage {
    pub content: String,
    pub permission: UpdatePermission,
}

// tell Holochain about the WikiPage
// entry definition
entry_defs!(WikiPage::entry_def());

#[derive(Serialize, Deserialize, SerializedBytes, Debug, Clone)]
pub struct UpdateWikiPage {
    wiki_page: WikiPage,
    address_to_update: HeaderHash,
}

#[hdk_extern]
pub fn add_wiki_page(_input: WikiPage) -> ExternResult<HeaderHash> {
    todo!();
}

#[hdk_extern]
pub fn update_wiki_page(_input: UpdateWikiPage) -> ExternResult<HeaderHash> {
    todo!();
}

// context for the calling of this function
#[hdk_extern]
fn validate_update_entry_wiki_page(
    validate_data: ValidateData,
) -> ExternResult<ValidateCallbackResult> {
    todo!();
}

#[cfg(test)]
pub mod tests {
    // means we can reference structs and functions from
    // the main scope of the file (update_wiki_page, UpdateWikiPage, etc.)
    use super::*;
    // imports from the fixt crate for Holochain test fixtures
    // used in testing modes
    use ::fixt::prelude::fixt;
    use ::fixt::prelude::paste;
    use ::fixt::prelude::Unpredictable;
    // import a "Fixturator" for HeaderHash
    // which allows us to spontaneously generate a new random HeaderHash
    // for the sake of testing
    use hdk::prelude::HeaderHashFixturator;
    use holochain_types::prelude::ElementFixturator;
    use holochain_types::prelude::ValidateDataFixturator;

    // a helper function used in the test scenarios
    fn setup_create_expectation(entry_with_def_id: EntryWithDefId) -> HeaderHash {
        let header_hash = fixt!(HeaderHash);
        let closure_header_hash = header_hash.clone();
        let mut mock_hdk = MockHdkT::new();
        mock_hdk
            .expect_create()
            .with(mockall::predicate::eq(entry_with_def_id))
            .times(1)
            .return_const(Ok(closure_header_hash));
        set_hdk(mock_hdk);
        header_hash
    }
    // a helper function used in the test scenarios
    fn setup_update_expectation(
        entry_with_def_id: EntryWithDefId,
        header_hash: HeaderHash,
    ) -> HeaderHash {
        let new_header_hash = fixt!(HeaderHash);
        let new_closure_header_hash = new_header_hash.clone();
        let mut mock_hdk = MockHdkT::new();
        mock_hdk
            .expect_update()
            .with(mockall::predicate::eq(UpdateInput::new(
                header_hash,
                entry_with_def_id,
            )))
            .times(1)
            .return_const(Ok(new_closure_header_hash));
        set_hdk(mock_hdk);
        new_header_hash
    }
    // a helper function used in the test scenarios
    fn setup_get_expectation(maybe_element: Option<Element>) {
        let mut mock_hdk = MockHdkT::new();
        mock_hdk
            .expect_get()
            .times(1)
            // act as if not present / not found
            .return_const(Ok(maybe_element));
        set_hdk(mock_hdk);
    }

    /*
      This test describes a scenario of a user calling the add_wiki_page
      API function with a new wiki page they want to commit
      and verifies that it gets committed to the source chain
    */
    #[test]
    fn test_add_wiki_page() {
        // prepare a fake wiki page to be committed
        // to a source chain
        let beluga_whale_wiki_page: WikiPage = WikiPage {
            content: String::from("Beluga whales are wonderful creatures of the sea"),
            permission: UpdatePermission::AuthorOnly,
        };

        // Set up a mock that expects an entry to be committed
        // and agrees to return a HeaderHash for that committed entry,
        // just like Holochain would do
        let header_hash =
            setup_create_expectation(EntryWithDefId::try_from(&beluga_whale_wiki_page).unwrap());

        // call the add_wiki_page Zome API Function
        // with the beluga whale wiki page as an input
        let zome_call_result = super::add_wiki_page(beluga_whale_wiki_page);

        // check that a HeaderHash was returned, indicating that
        // a new WikiPage entry was committed to the source chain!
        assert_eq!(zome_call_result, Ok(header_hash),);
    }

    /*
      This test describes a scenario of a user calling the update_wiki_page
      API function with the HeaderHash of an WikiPage entry to update,
      and the new WikiPage contents.
    */
    #[test]
    fn test_update_wiki_page() {
        // prepare fake wiki page details
        // as if it were to be updated content for an existing page
        let orca_whale_wiki_page: WikiPage = WikiPage {
            content: String::from("Orca whales swim the ocean"),
            permission: UpdatePermission::AuthorOnly,
        };
        // make up a HeaderHash for an imaginary
        // existing entry to update, and assume it exists
        let address_to_update = fixt!(HeaderHash);
        let input: UpdateWikiPage = UpdateWikiPage {
            wiki_page: orca_whale_wiki_page.clone(),
            address_to_update: address_to_update.clone(),
        };

        // Set up a mock that expects an entry to be committed
        // and agrees to return a HeaderHash for that committed entry,
        // just like Holochain would do
        let header_hash = setup_update_expectation(
            EntryWithDefId::try_from(&orca_whale_wiki_page).unwrap(),
            address_to_update,
        );

        // call the update_wiki_page Zome API Function
        // with the beluga whale wiki page as an input
        let zome_call_result = super::update_wiki_page(input);

        // check that a HeaderHash was returned, indicating that
        // an update to the original WikiPage entry was committed to the source chain!
        assert_eq!(zome_call_result, Ok(header_hash),);
    }

    /*
      Validation Rules:
      If the original author set
      the `permission` field to UpdatePermission::AuthorOnly then someone other than the
      original author trying to update it will fail validation
    */
    #[test]
    fn test_validate_locked_wiki_page() {
        // Provide the "Original" Wiki Page
        let locked_wiki_page: WikiPage = WikiPage {
            content: String::from("Beluga whales are wonderful creatures of the sea"),
            permission: UpdatePermission::AuthorOnly,
        };
        // this will generate a random author/agent for the header
        let mut original_element = fixt!(Element);
        *original_element.as_entry_mut() =
            ElementEntry::Present(locked_wiki_page.clone().try_into().unwrap());
        // whenever `get` is called, it will return the
        // original WikiPage, nested in this element
        setup_get_expectation(Some(original_element));

        // Provide the new "updated" WikiPage
        // and try to validate it, assuming that
        // it will refer back to the "original" as
        // being what it updates
        let updated_wiki_page: WikiPage = WikiPage {
            content: String::from("Beluga whales are silly creatures of the sea"),
            permission: UpdatePermission::AuthorOnly,
        };
        // this will generate a random author/agent of the header
        let mut validate_data = fixt!(ValidateData);
        let update_header = fixt!(Update);
        *validate_data.element.as_header_mut() = Header::Update(update_header.clone());
        *validate_data.element.as_entry_mut() =
            ElementEntry::Present(updated_wiki_page.clone().try_into().unwrap());

        assert_eq!(
            validate_update_entry_wiki_page(validate_data),
            Ok(ValidateCallbackResult::Invalid(String::from(
                "only the author can edit this WikiPage",
            ))),
        );
    }

    /*
      Validation Rules:
      If the original author set
      the `permission` field to UpdatePermission::Others then they
      or anyone else can edit it
    */
    #[test]
    fn test_validate_unlocked_wiki_page() {
        // Provide the "Original" Wiki Page
        let locked_wiki_page: WikiPage = WikiPage {
            content: String::from("Beluga whales are wonderful creatures of the sea"),
            permission: UpdatePermission::Others,
        };
        // this will generate a random author/agent for the header
        let mut original_element = fixt!(Element);
        *original_element.as_entry_mut() =
            ElementEntry::Present(locked_wiki_page.clone().try_into().unwrap());
        // whenever `get` is called, it will return the
        // original WikiPage, nested in this element
        setup_get_expectation(Some(original_element.clone()));

        // Provide the new "updated" WikiPage
        // and try to validate it, assuming that
        // it will refer back to the "original" as
        // being what it updates
        let updated_wiki_page: WikiPage = WikiPage {
            content: String::from("Beluga whales are silly creatures of the sea"),
            permission: UpdatePermission::AuthorOnly,
        };
        // this will generate a random author/agent of the header
        let mut validate_data = fixt!(ValidateData);
        let update_header = fixt!(Update);
        *validate_data.element.as_header_mut() = Header::Update(update_header.clone());
        *validate_data.element.as_entry_mut() =
            ElementEntry::Present(updated_wiki_page.clone().try_into().unwrap());

        assert_eq!(
            validate_update_entry_wiki_page(validate_data),
            Ok(ValidateCallbackResult::Valid),
        );
    }
}
