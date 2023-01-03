use std::{path::Path, sync::{Mutex, Arc}};

use crate::{
      data_model::{
      Bucket, 
      Ticket, 
      State, 
      Tag, 
      Filter, 
      Config
   }, 
   AppConfig
};

pub use super::adapter_error::AdapterError as AdapterError;

pub trait TicketAdapter {

    /**
       Returns the generic type name, that is used to identify it's type from config
    */
    fn get_type_name() -> String where Self: Sized;

    /**
       Returns the fancy generic Type name, that is displayed before instantiating the Adapter
    */
    fn get_fancy_type_name() -> String where Self: Sized;

    /**
       Returns the technical name of the ticket adapter. This name is
       used in commands and expressions
     */
    fn get_name(&self) -> String;

    /**
       Returns a reference Configuration needed for instantiating the Ticket Adapter
     */
    fn create_config() -> Config where Self: Sized;

    /**
       Creates an instance from Configuration
     */
    fn from_config(app_config: Arc<Mutex<AppConfig>>, config: &Config) -> Result<Box<dyn TicketAdapter + Send + Sync>, AdapterError> where Self: Sized;

    /**
       Returns the path to the icon for this particular adapter.
     */
    fn get_icon(&self) -> &Path;

    /**
       Returns the Human readable name, this is the Name, that gets
       displayed in the USer interface
     */
    fn get_fancy_name(&self) -> String;

    /**
       Returns a boolean on wether the Adapter is readonly or not.
       If it is readonly
     */
    fn is_read_only(&self) -> bool;

    /**
       Lists all Buckets, that are provided by this adapter
       If the operation fails, returns an empty vector
     */
    fn bucket_list_all(&self) -> Vec<Bucket>;

    /**
       Lists a single Bucket this adapter can provide, defned by it's id.
       if the read fails, the function returns None
     */
    fn bucket_list_unique(&self, id: i64) -> Option<Bucket>;

    /**
       Tries to delete a bucket off this adapter. If the delete fails for
       for whatever reason, an AdapterError is being thrown.
     */
    fn bucket_drop(&self, bucket: &Bucket) -> Result<(), AdapterError>;

    /**
       Writes a bucket to this adapter. If the Write fails, it will return
       an AdapterError, otherwise an empty tuple, that can be ignored
     */
    fn bucket_write(&self, bucket: &mut Bucket) -> Result<(), AdapterError>;

    /**
       Lists all tickets, this adapter can provide. If the read fails, or
       no tickets are available, an empty vector will be returned
     */
    fn ticket_list_all(&self) -> Vec<Ticket>;


    /**
       Lists a single Ticket this adapter can provide, defned by it's id.
       if the read fails, the function returns None
     */
    fn ticket_list_unique(&self, id: i64) -> Option<Ticket>;

    /**
       Lists tickets, that the adapter can provide based on an adapter
       specific expression. If this expression is invalid, an error will
       be thrown, a vector with (or without) tickets otherwise
     */
    fn ticket_list(&self, expression: &str) -> Result<Vec<Ticket>, AdapterError>;

    /**
       Tries to write a ticket to this adapter. If the write fails, it
       throw an AdapterError.
     */
    fn ticket_write(&self, ticket: &Ticket) -> Result<(), AdapterError>;

    /**
       Tries to delete a ticket off this adapter. If the delete fails for
       for whatever reason, an AdapterError is being thrown.
     */
    fn ticket_drop(&self, ticket: &Ticket) -> Result<(), AdapterError>;

    /**
       Lists all states in a list, that are available to this adapter
       or an empty list
     */
    fn state_list_all(&self) -> Vec<State>;

    /**
       Writes a state to the adapter. if the write failed, then an
       AdapterError will be thrown. On Success it returns an empty
       Tuple
     */
    fn state_write(&self, state: &State) -> Result<(), AdapterError>;

    /**
       Instructs the adapter to list all available tags.
       If any tags are available, then it will return a vector with
       all of them, an empty one otherwise
     */
    fn tag_list_all(&self) -> Vec<Tag>;

    /**
       Attempts to write a tag to the adapter. It the write fails,
       an error gets thrown. Otherwise an empty tuple gets returned
     */
    fn tag_write(&self, state: &Tag) -> Result<(), AdapterError>;

    /**
       Tries to remove a tag from this adapter. If the delete
       operation fails for whatever reason, an error gets thrown.
     */
    fn tag_drop(&self, state: &Tag) -> Result<(), AdapterError>;
    
    /**
       Lists all available filters in this adapter as a vector,
       returns an empty vector, if no filters have been found
     */
    fn filter_list_all(&self) -> Vec<Filter>;

    /**
       Get a filter from the adapter, that has the same name as the 
       supplied string. If no Filter is found, None is returned.
     */
    fn filter_list(&self, filter_name: String) -> Option<Filter>;

    /**
       Attempts to write a filter to the adapter. If there is an Error,
       a AdapterError gets thrown.
     */
    fn filter_write(&self, filter: &Filter) -> Result<(), AdapterError>;

    /**
       Tries to delete a filter off this adapter. If the delete fails for
       for whatever reason, an AdapterError is being thrown.
     */
    fn filter_drop(&self, filter: &Filter) -> Result<(), AdapterError>;

    /**
       Tests, if a filter expression is valid for this specific adapter. If there is
       a problem, returns a List of errors in the form of Vec<(Attribute Name, Error Message)>,
       otherwise returns an Empty Tuple.
     */
    fn filter_expression_validate(&self, expression: &String) -> Result<(), Vec<(String, String)>>;
}