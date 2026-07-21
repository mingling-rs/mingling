#![feature(prelude_import)]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use crate::{linter::MinglingLinterSetup, metadata::MinglingMetadataSetup};
use mingling::{
    macros::gen_program,
    setup::{ExitCodeSetup, picker::{HelpFlagSetup, StructuralRendererSetup}},
};
pub mod errors {
    pub mod serde_json {
        use mingling::macros::{buffer, group, r_println, renderer};
        pub type ErrorSerdeJson = serde_json::Error;
        #[allow(non_camel_case_types)]
        mod internal_group_serde_json_error {
            use crate::ThisProgram as __MinglingProgram;
            #[allow(unused_imports)]
            use serde_json::Error;
            use super::ErrorSerdeJson;
            impl ::mingling::Grouped<__MinglingProgram> for ErrorSerdeJson {
                fn member_id() -> __MinglingProgram {
                    __MinglingProgram::ErrorSerdeJson
                }
            }
        }
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        pub struct __internal_renderer_render_error_serde_json;
        impl ::mingling::Renderer for __internal_renderer_render_error_serde_json {
            type Previous = ErrorSerdeJson;
            fn render(prev: Self::Previous) -> ::mingling::RenderResult {
                let __renderer_result = { render_error_serde_json(prev) };
                ::std::convert::Into::into(__renderer_result)
            }
        }
        pub fn render_error_serde_json(
            _err: ErrorSerdeJson,
        ) -> ::mingling::RenderResult {
            let mut __render_result_buffer = ::mingling::RenderResult::new();
            {
                __render_result_buffer
                    .println(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(format_args!("serde"))
                        }),
                    );
            }
            __render_result_buffer
        }
    }
}
pub mod linter {
    use mingling::{Program, macros::program_setup};
    use crate::linter::cmd_mlint::CMDMinglingLinter;
    pub mod cmd_mlint {
        use mingling::macros::{chain, dispatcher};
        pub struct CMDMinglingLinter;
        #[automatically_derived]
        impl ::core::fmt::Debug for CMDMinglingLinter {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "CMDMinglingLinter")
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for CMDMinglingLinter {
            #[inline]
            fn default() -> CMDMinglingLinter {
                CMDMinglingLinter {}
            }
        }
        pub struct EntryMinglingLinter {
            pub(crate) inner: Vec<String>,
        }
        impl EntryMinglingLinter {
            /// Creates a new instance of the wrapper type
            pub fn new(inner: Vec<String>) -> Self {
                Self { inner }
            }
        }
        impl From<Vec<String>> for EntryMinglingLinter {
            fn from(inner: Vec<String>) -> Self {
                Self::new(inner)
            }
        }
        impl From<EntryMinglingLinter> for Vec<String> {
            fn from(wrapper: EntryMinglingLinter) -> Vec<String> {
                wrapper.inner
            }
        }
        impl ::std::convert::AsRef<Vec<String>> for EntryMinglingLinter {
            fn as_ref(&self) -> &Vec<String> {
                &self.inner
            }
        }
        impl ::std::convert::AsMut<Vec<String>> for EntryMinglingLinter {
            fn as_mut(&mut self) -> &mut Vec<String> {
                &mut self.inner
            }
        }
        impl ::std::ops::Deref for EntryMinglingLinter {
            type Target = Vec<String>;
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }
        impl ::std::ops::DerefMut for EntryMinglingLinter {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.inner
            }
        }
        impl ::std::default::Default for EntryMinglingLinter
        where
            Vec<String>: ::std::default::Default,
        {
            fn default() -> Self {
                Self::new(::std::default::Default::default())
            }
        }
        impl Into<mingling::AnyOutput<crate::ThisProgram>> for EntryMinglingLinter {
            fn into(self) -> mingling::AnyOutput<crate::ThisProgram> {
                mingling::AnyOutput::new(self)
            }
        }
        impl Into<mingling::ChainProcess<crate::ThisProgram>> for EntryMinglingLinter {
            fn into(self) -> mingling::ChainProcess<crate::ThisProgram> {
                mingling::AnyOutput::new(self).route_chain()
            }
        }
        impl ::mingling::Grouped<crate::ThisProgram> for EntryMinglingLinter {
            fn member_id() -> crate::ThisProgram {
                crate::ThisProgram::EntryMinglingLinter
            }
        }
        impl ::mingling::Dispatcher<crate::ThisProgram> for CMDMinglingLinter {
            fn node(&self) -> ::mingling::Node {
                mingling::Node::default().join("mlint")
            }
            fn begin(
                &self,
                args: Vec<String>,
            ) -> ::mingling::ChainProcess<crate::ThisProgram> {
                use ::mingling::Grouped;
                ::mingling::Routable::to_chain(EntryMinglingLinter::new(args))
            }
            fn clone_dispatcher(
                &self,
            ) -> Box<dyn ::mingling::Dispatcher<crate::ThisProgram>> {
                Box::new(CMDMinglingLinter)
            }
        }
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        pub struct __internal_chain_handle_mlint;
        impl ::mingling::Chain<crate::ThisProgram> for __internal_chain_handle_mlint {
            type Previous = EntryMinglingLinter;
            fn proc(
                prev: EntryMinglingLinter,
            ) -> ::mingling::ChainProcess<crate::ThisProgram> {
                handle_mlint(prev);
                <crate::ResultEmpty as ::mingling::Routable<
                    crate::ThisProgram,
                >>::to_chain(crate::ResultEmpty)
            }
        }
        pub fn handle_mlint(_args: EntryMinglingLinter) {}
    }
    #[doc(hidden)]
    pub struct MinglingLinterSetup;
    impl ::mingling::setup::ProgramSetup<crate::ThisProgram> for MinglingLinterSetup {
        fn setup(self, program: &mut ::mingling::Program<crate::ThisProgram>) {
            mingling_linter_setup(program);
        }
    }
    pub fn mingling_linter_setup(program: &mut Program<crate::ThisProgram>) {
        {
            program.with_setup(MinglingLinterCommandSetup);
        }
    }
    #[doc(hidden)]
    pub struct MinglingLinterCommandSetup;
    impl ::mingling::setup::ProgramSetup<crate::ThisProgram>
    for MinglingLinterCommandSetup {
        fn setup(self, program: &mut ::mingling::Program<crate::ThisProgram>) {
            mingling_linter_command_setup(program);
        }
    }
    pub fn mingling_linter_command_setup(program: &mut Program<crate::ThisProgram>) {
        {
            program.with_dispatcher(CMDMinglingLinter);
        }
    }
}
pub mod metadata {
    use cargo_metadata::Metadata;
    use mingling::{
        AnyOutput, Program, ProgramCollect, RenderResult,
        macros::{buffer, group_structural, program_setup, r_println, renderer, routeify},
    };
    use crate::metadata::{cmd_metadata::CMDMetadata, setup::CargoMetadataSetup};
    pub mod cmd_metadata {
        use cargo_metadata::Metadata;
        use mingling::{LazyRes, macros::{chain, dispatcher, pack_structural}};
        use crate::metadata::setup::ResMetadata;
        pub struct CMDMetadata;
        #[automatically_derived]
        impl ::core::fmt::Debug for CMDMetadata {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "CMDMetadata")
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for CMDMetadata {
            #[inline]
            fn default() -> CMDMetadata {
                CMDMetadata {}
            }
        }
        pub struct EntryMetadata {
            pub(crate) inner: Vec<String>,
        }
        impl EntryMetadata {
            /// Creates a new instance of the wrapper type
            pub fn new(inner: Vec<String>) -> Self {
                Self { inner }
            }
        }
        impl From<Vec<String>> for EntryMetadata {
            fn from(inner: Vec<String>) -> Self {
                Self::new(inner)
            }
        }
        impl From<EntryMetadata> for Vec<String> {
            fn from(wrapper: EntryMetadata) -> Vec<String> {
                wrapper.inner
            }
        }
        impl ::std::convert::AsRef<Vec<String>> for EntryMetadata {
            fn as_ref(&self) -> &Vec<String> {
                &self.inner
            }
        }
        impl ::std::convert::AsMut<Vec<String>> for EntryMetadata {
            fn as_mut(&mut self) -> &mut Vec<String> {
                &mut self.inner
            }
        }
        impl ::std::ops::Deref for EntryMetadata {
            type Target = Vec<String>;
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }
        impl ::std::ops::DerefMut for EntryMetadata {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.inner
            }
        }
        impl ::std::default::Default for EntryMetadata
        where
            Vec<String>: ::std::default::Default,
        {
            fn default() -> Self {
                Self::new(::std::default::Default::default())
            }
        }
        impl Into<mingling::AnyOutput<crate::ThisProgram>> for EntryMetadata {
            fn into(self) -> mingling::AnyOutput<crate::ThisProgram> {
                mingling::AnyOutput::new(self)
            }
        }
        impl Into<mingling::ChainProcess<crate::ThisProgram>> for EntryMetadata {
            fn into(self) -> mingling::ChainProcess<crate::ThisProgram> {
                mingling::AnyOutput::new(self).route_chain()
            }
        }
        impl ::mingling::Grouped<crate::ThisProgram> for EntryMetadata {
            fn member_id() -> crate::ThisProgram {
                crate::ThisProgram::EntryMetadata
            }
        }
        impl ::mingling::Dispatcher<crate::ThisProgram> for CMDMetadata {
            fn node(&self) -> ::mingling::Node {
                mingling::Node::default().join("metadata")
            }
            fn begin(
                &self,
                args: Vec<String>,
            ) -> ::mingling::ChainProcess<crate::ThisProgram> {
                use ::mingling::Grouped;
                ::mingling::Routable::to_chain(EntryMetadata::new(args))
            }
            fn clone_dispatcher(
                &self,
            ) -> Box<dyn ::mingling::Dispatcher<crate::ThisProgram>> {
                Box::new(CMDMetadata)
            }
        }
        pub struct ResultMetadata {
            pub inner: ResMetadata,
        }
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for ResultMetadata {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private229::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    let mut __serde_state = _serde::Serializer::serialize_struct(
                        __serializer,
                        "ResultMetadata",
                        false as usize + 1,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "inner",
                        &self.inner,
                    )?;
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        impl ResultMetadata {
            pub fn new(inner: ResMetadata) -> Self {
                Self { inner }
            }
        }
        impl From<ResMetadata> for ResultMetadata {
            fn from(inner: ResMetadata) -> Self {
                Self::new(inner)
            }
        }
        impl From<ResultMetadata> for ResMetadata {
            fn from(wrapper: ResultMetadata) -> ResMetadata {
                wrapper.inner
            }
        }
        impl ::std::convert::AsRef<ResMetadata> for ResultMetadata {
            fn as_ref(&self) -> &ResMetadata {
                &self.inner
            }
        }
        impl ::std::convert::AsMut<ResMetadata> for ResultMetadata {
            fn as_mut(&mut self) -> &mut ResMetadata {
                &mut self.inner
            }
        }
        impl ::std::ops::Deref for ResultMetadata {
            type Target = ResMetadata;
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }
        impl ::std::ops::DerefMut for ResultMetadata {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.inner
            }
        }
        impl ::std::default::Default for ResultMetadata
        where
            ResMetadata: ::std::default::Default,
        {
            fn default() -> Self {
                Self::new(::std::default::Default::default())
            }
        }
        impl ::mingling::__private::StructuralDataSealed<crate::ThisProgram>
        for ResultMetadata {}
        impl ::mingling::__private::StructuralData<crate::ThisProgram>
        for ResultMetadata {}
        impl Into<::mingling::AnyOutput<crate::ThisProgram>> for ResultMetadata {
            fn into(self) -> ::mingling::AnyOutput<crate::ThisProgram> {
                ::mingling::AnyOutput::new(self)
            }
        }
        impl Into<::mingling::ChainProcess<crate::ThisProgram>> for ResultMetadata {
            fn into(self) -> ::mingling::ChainProcess<crate::ThisProgram> {
                ::mingling::AnyOutput::new(self).route_chain()
            }
        }
        impl ::mingling::Grouped<crate::ThisProgram> for ResultMetadata {
            fn member_id() -> crate::ThisProgram {
                crate::ThisProgram::ResultMetadata
            }
        }
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        pub struct __internal_chain_handle_metadata;
        impl ::mingling::Chain<crate::ThisProgram> for __internal_chain_handle_metadata {
            type Previous = EntryMetadata;
            fn proc(
                prev: EntryMetadata,
            ) -> ::mingling::ChainProcess<crate::ThisProgram> {
                let __chain_result = {
                    ::mingling::this::<crate::ThisProgram>()
                        .__modify_res_and_return_route(|
                            metadata: &mut LazyRes<ResMetadata>|
                        {
                            ::mingling::Routable::<
                                crate::ThisProgram,
                            >::to_chain(handle_metadata(prev, metadata))
                        })
                };
                ::mingling::Routable::<crate::ThisProgram>::to_chain(__chain_result)
            }
        }
        pub fn handle_metadata(
            _: EntryMetadata,
            metadata: &mut LazyRes<ResMetadata>,
        ) -> Metadata {
            let metadata = metadata.get_ref().clone();
            metadata.data().clone()
        }
    }
    pub mod setup {
        use std::{env::current_dir, path::PathBuf};
        use cargo_metadata::{CargoOpt, MetadataCommand};
        use mingling::{
            LazyInit, Program, consts::REMAINS, macros::program_setup,
            picker::{IntoPicker, value::Flag},
            prelude::*, res::ResCurrentDir,
        };
        /// Name of Cargo manifest file
        pub const CARGO_TOML: &str = "Cargo.toml";
        use cargo_metadata::Metadata;
        use serde::Serialize;
        pub struct ResMetadata {
            data: Option<Metadata>,
        }
        #[automatically_derived]
        impl ::core::default::Default for ResMetadata {
            #[inline]
            fn default() -> ResMetadata {
                ResMetadata {
                    data: ::core::default::Default::default(),
                }
            }
        }
        #[automatically_derived]
        impl ::core::clone::Clone for ResMetadata {
            #[inline]
            fn clone(&self) -> ResMetadata {
                ResMetadata {
                    data: ::core::clone::Clone::clone(&self.data),
                }
            }
        }
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            clippy::absolute_paths,
        )]
        const _: () = {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
            #[automatically_derived]
            impl _serde::Serialize for ResMetadata {
                fn serialize<__S>(
                    &self,
                    __serializer: __S,
                ) -> _serde::__private229::Result<__S::Ok, __S::Error>
                where
                    __S: _serde::Serializer,
                {
                    let mut __serde_state = _serde::Serializer::serialize_struct(
                        __serializer,
                        "ResMetadata",
                        false as usize + 1,
                    )?;
                    _serde::ser::SerializeStruct::serialize_field(
                        &mut __serde_state,
                        "data",
                        &self.data,
                    )?;
                    _serde::ser::SerializeStruct::end(__serde_state)
                }
            }
        };
        impl ResMetadata {
            /// Returns a reference to the parsed `cargo metadata`.
            ///
            /// # Panics
            ///
            /// This function does **not** panic in practice, because `ResMetadata` is
            /// always initialized via [`LazyInit`] inside the program setup, and the
            /// initialization either succeeds (setting `data` to `Some(...)`) or fails
            /// by propagating the error from `cmd.exec().unwrap()`. Therefore, by the
            /// time this getter is called, `self.data` is guaranteed to be `Some`.
            pub fn data(&self) -> &Metadata {
                self.data.as_ref().unwrap()
            }
        }
        #[doc(hidden)]
        pub struct CargoMetadataSetup;
        impl ::mingling::setup::ProgramSetup<crate::ThisProgram> for CargoMetadataSetup {
            fn setup(self, program: &mut ::mingling::Program<crate::ThisProgram>) {
                cargo_metadata_setup(program);
            }
        }
        pub fn cargo_metadata_setup(program: &mut Program<crate::ThisProgram>) {
            {
                let args = program.take_args();
                let (
                    feature_args,
                    manifest_path,
                    message_format,
                    enable_all_features,
                    no_default_features,
                    no_deps,
                    args,
                ) = args
                    .pick(
                        &::mingling::picker::PickerArg::<Vec<String>> {
                            full: &["features"],
                            short: ::std::option::Option::None,
                            positional: false,
                            internal_type: ::std::marker::PhantomData,
                        },
                    )
                    .pick(
                        &::mingling::picker::PickerArg::<Option<String>> {
                            full: &["manifest_path"],
                            short: ::std::option::Option::None,
                            positional: false,
                            internal_type: ::std::marker::PhantomData,
                        },
                    )
                    .pick_or(
                        &::mingling::picker::PickerArg::<String> {
                            full: &["message_format"],
                            short: ::std::option::Option::None,
                            positional: false,
                            internal_type: ::std::marker::PhantomData,
                        },
                        || "disable".to_string(),
                    )
                    .pick(
                        &::mingling::picker::PickerArg::<Flag> {
                            full: &["all_features"],
                            short: ::std::option::Option::None,
                            positional: false,
                            internal_type: ::std::marker::PhantomData,
                        },
                    )
                    .pick(
                        &::mingling::picker::PickerArg::<Flag> {
                            full: &["no_default_features"],
                            short: ::std::option::Option::None,
                            positional: false,
                            internal_type: ::std::marker::PhantomData,
                        },
                    )
                    .pick(
                        &::mingling::picker::PickerArg::<Flag> {
                            full: &["no_deps"],
                            short: ::std::option::Option::None,
                            positional: false,
                            internal_type: ::std::marker::PhantomData,
                        },
                    )
                    .pick(&REMAINS)
                    .unwrap();
                program.replace_args(args.into());
                program.structural_renderer_name = message_format.into();
                let current_dir = current_dir().unwrap();
                let feature_args_leaked: &[String] = Box::leak(
                    feature_args.into_boxed_slice(),
                );
                let manifest_path_clone = manifest_path.clone();
                let current_dir_clone = current_dir.clone();
                program
                    .with_resource(
                        ResMetadata::lazy_init(move || {
                            let metadata_path = match manifest_path_clone {
                                Some(ref path_str) => PathBuf::from(path_str),
                                None => find_manifest().unwrap(),
                            };
                            let mut cmd = MetadataCommand::new();
                            cmd.manifest_path(metadata_path)
                                .current_dir(&current_dir_clone);
                            if *enable_all_features {
                                cmd.features(CargoOpt::AllFeatures);
                            }
                            if *no_default_features {
                                cmd.features(CargoOpt::NoDefaultFeatures);
                            }
                            if *no_deps {
                                cmd.no_deps();
                            }
                            cmd.features(
                                CargoOpt::SomeFeatures(feature_args_leaked.to_vec()),
                            );
                            ResMetadata {
                                data: Some(cmd.exec().unwrap()),
                            }
                        }),
                    );
                program.with_resource(ResCurrentDir::from(current_dir));
            }
        }
        /// Find `Cargo.toml` by searching upward from `current_dir`.
        fn find_manifest() -> Option<PathBuf> {
            let mut dir = current_dir().ok()?;
            loop {
                let candidate = dir.join(CARGO_TOML);
                if candidate.is_file() {
                    return Some(candidate);
                }
                if !dir.pop() {
                    return None;
                }
            }
        }
    }
    #[doc(hidden)]
    pub struct MinglingMetadataSetup;
    impl ::mingling::setup::ProgramSetup<crate::ThisProgram> for MinglingMetadataSetup {
        fn setup(self, program: &mut ::mingling::Program<crate::ThisProgram>) {
            mingling_metadata_setup(program);
        }
    }
    pub fn mingling_metadata_setup(program: &mut Program<crate::ThisProgram>) {
        {
            program.with_setup(CargoMetadataSetup);
            program.with_dispatcher(CMDMetadata);
        }
    }
    #[allow(non_camel_case_types)]
    mod internal_group_metadata {
        use crate::ThisProgram as __MinglingProgram;
        #[allow(unused_imports)]
        use super::Metadata;
        impl ::mingling::Grouped<__MinglingProgram> for Metadata {
            fn member_id() -> __MinglingProgram {
                __MinglingProgram::Metadata
            }
        }
        impl ::mingling::__private::StructuralDataSealed<crate::ThisProgram>
        for Metadata {}
        impl ::mingling::__private::StructuralData<crate::ThisProgram> for Metadata {}
    }
    #[doc(hidden)]
    #[allow(non_camel_case_types)]
    pub struct __internal_renderer_render_metadata;
    impl ::mingling::Renderer for __internal_renderer_render_metadata {
        type Previous = Metadata;
        fn render(prev: Self::Previous) -> ::mingling::RenderResult {
            let __renderer_result = { render_metadata(prev) };
            ::std::convert::Into::into(__renderer_result)
        }
    }
    pub fn render_metadata(metadata: Metadata) -> RenderResult {
        let mut r = RenderResult::new();
        r.println(
            ::alloc::__export::must_use({
                ::alloc::fmt::format(
                    format_args!("{0}", serde_json::to_string(&metadata)?),
                )
            }),
        );
        r
    }
}
fn main() {
    let mut program = ThisProgram::new();
    program.with_setup(HelpFlagSetup::default());
    program.with_setup(ExitCodeSetup::default());
    program.with_setup(StructuralRendererSetup);
    program.with_setup(MinglingMetadataSetup);
    program.with_setup(MinglingLinterSetup);
    program.exec_and_exit();
}
/// Alias for the current program type `crate::ThisProgram`
pub type Next = ::mingling::ChainProcess<crate::ThisProgram>;
impl ::mingling::Routable<crate::ThisProgram>
for ::mingling::ChainProcess<crate::ThisProgram> {
    fn to_chain(self) -> ::mingling::ChainProcess<crate::ThisProgram> {
        match self {
            ::mingling::ChainProcess::Ok((any, _)) => {
                ::mingling::ChainProcess::Ok((any, mingling::NextProcess::Chain))
            }
            other => other,
        }
    }
    fn to_render(self) -> ::mingling::ChainProcess<crate::ThisProgram> {
        match self {
            ::mingling::ChainProcess::Ok((any, _)) => {
                ::mingling::ChainProcess::Ok((any, mingling::NextProcess::Renderer))
            }
            other => other,
        }
    }
}
pub struct ErrorRendererNotFound {
    pub(crate) inner: String,
}
impl ErrorRendererNotFound {
    /// Creates a new instance of the wrapper type
    pub fn new(inner: String) -> Self {
        Self { inner }
    }
}
impl From<String> for ErrorRendererNotFound {
    fn from(inner: String) -> Self {
        Self::new(inner)
    }
}
impl From<ErrorRendererNotFound> for String {
    fn from(wrapper: ErrorRendererNotFound) -> String {
        wrapper.inner
    }
}
impl ::std::convert::AsRef<String> for ErrorRendererNotFound {
    fn as_ref(&self) -> &String {
        &self.inner
    }
}
impl ::std::convert::AsMut<String> for ErrorRendererNotFound {
    fn as_mut(&mut self) -> &mut String {
        &mut self.inner
    }
}
impl ::std::ops::Deref for ErrorRendererNotFound {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl ::std::ops::DerefMut for ErrorRendererNotFound {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
impl ::std::default::Default for ErrorRendererNotFound
where
    String: ::std::default::Default,
{
    fn default() -> Self {
        Self::new(::std::default::Default::default())
    }
}
impl Into<mingling::AnyOutput<crate::ThisProgram>> for ErrorRendererNotFound {
    fn into(self) -> mingling::AnyOutput<crate::ThisProgram> {
        mingling::AnyOutput::new(self)
    }
}
impl Into<mingling::ChainProcess<crate::ThisProgram>> for ErrorRendererNotFound {
    fn into(self) -> mingling::ChainProcess<crate::ThisProgram> {
        mingling::AnyOutput::new(self).route_chain()
    }
}
impl ::mingling::Grouped<crate::ThisProgram> for ErrorRendererNotFound {
    fn member_id() -> crate::ThisProgram {
        crate::ThisProgram::ErrorRendererNotFound
    }
}
pub struct ErrorDispatcherNotFound {
    pub(crate) inner: Vec<String>,
}
impl ErrorDispatcherNotFound {
    /// Creates a new instance of the wrapper type
    pub fn new(inner: Vec<String>) -> Self {
        Self { inner }
    }
}
impl From<Vec<String>> for ErrorDispatcherNotFound {
    fn from(inner: Vec<String>) -> Self {
        Self::new(inner)
    }
}
impl From<ErrorDispatcherNotFound> for Vec<String> {
    fn from(wrapper: ErrorDispatcherNotFound) -> Vec<String> {
        wrapper.inner
    }
}
impl ::std::convert::AsRef<Vec<String>> for ErrorDispatcherNotFound {
    fn as_ref(&self) -> &Vec<String> {
        &self.inner
    }
}
impl ::std::convert::AsMut<Vec<String>> for ErrorDispatcherNotFound {
    fn as_mut(&mut self) -> &mut Vec<String> {
        &mut self.inner
    }
}
impl ::std::ops::Deref for ErrorDispatcherNotFound {
    type Target = Vec<String>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl ::std::ops::DerefMut for ErrorDispatcherNotFound {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
impl ::std::default::Default for ErrorDispatcherNotFound
where
    Vec<String>: ::std::default::Default,
{
    fn default() -> Self {
        Self::new(::std::default::Default::default())
    }
}
impl Into<mingling::AnyOutput<crate::ThisProgram>> for ErrorDispatcherNotFound {
    fn into(self) -> mingling::AnyOutput<crate::ThisProgram> {
        mingling::AnyOutput::new(self)
    }
}
impl Into<mingling::ChainProcess<crate::ThisProgram>> for ErrorDispatcherNotFound {
    fn into(self) -> mingling::ChainProcess<crate::ThisProgram> {
        mingling::AnyOutput::new(self).route_chain()
    }
}
impl ::mingling::Grouped<crate::ThisProgram> for ErrorDispatcherNotFound {
    fn member_id() -> crate::ThisProgram {
        crate::ThisProgram::ErrorDispatcherNotFound
    }
}
pub struct ResultEmpty;
#[doc(hidden)]
#[allow(
    non_upper_case_globals,
    unused_attributes,
    unused_qualifications,
    clippy::absolute_paths,
)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for ResultEmpty {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private229::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            _serde::Serializer::serialize_unit_struct(__serializer, "ResultEmpty")
        }
    }
};
impl ::mingling::__private::StructuralDataSealed<crate::ThisProgram> for ResultEmpty {}
impl ::mingling::__private::StructuralData<crate::ThisProgram> for ResultEmpty {}
impl ::mingling::Grouped<crate::ThisProgram> for ResultEmpty {
    fn member_id() -> crate::ThisProgram {
        crate::ThisProgram::ResultEmpty
    }
}
impl ::std::convert::Into<::mingling::AnyOutput<crate::ThisProgram>> for ResultEmpty {
    fn into(self) -> ::mingling::AnyOutput<crate::ThisProgram> {
        ::mingling::AnyOutput::new(self)
    }
}
impl ::std::convert::Into<::mingling::ChainProcess<crate::ThisProgram>> for ResultEmpty {
    fn into(self) -> ::mingling::ChainProcess<crate::ThisProgram> {
        ::mingling::AnyOutput::new(self).route_chain()
    }
}
#[automatically_derived]
impl ::core::default::Default for ResultEmpty {
    #[inline]
    fn default() -> ResultEmpty {
        ResultEmpty {}
    }
}
#[repr(u8)]
#[allow(nonstandard_style)]
pub enum ThisProgram {
    EntryMetadata,
    EntryMinglingLinter,
    ErrorDispatcherNotFound,
    ErrorRendererNotFound,
    ErrorSerdeJson,
    Metadata,
    ResultEmpty,
    ResultMetadata,
}
#[automatically_derived]
#[allow(nonstandard_style)]
impl ::core::fmt::Debug for ThisProgram {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                ThisProgram::EntryMetadata => "EntryMetadata",
                ThisProgram::EntryMinglingLinter => "EntryMinglingLinter",
                ThisProgram::ErrorDispatcherNotFound => "ErrorDispatcherNotFound",
                ThisProgram::ErrorRendererNotFound => "ErrorRendererNotFound",
                ThisProgram::ErrorSerdeJson => "ErrorSerdeJson",
                ThisProgram::Metadata => "Metadata",
                ThisProgram::ResultEmpty => "ResultEmpty",
                ThisProgram::ResultMetadata => "ResultMetadata",
            },
        )
    }
}
#[automatically_derived]
#[allow(nonstandard_style)]
impl ::core::marker::StructuralPartialEq for ThisProgram {}
#[automatically_derived]
#[allow(nonstandard_style)]
impl ::core::cmp::PartialEq for ThisProgram {
    #[inline]
    fn eq(&self, other: &ThisProgram) -> bool {
        let __self_discr = ::core::intrinsics::discriminant_value(self);
        let __arg1_discr = ::core::intrinsics::discriminant_value(other);
        __self_discr == __arg1_discr
    }
}
#[automatically_derived]
#[allow(nonstandard_style)]
impl ::core::cmp::Eq for ThisProgram {
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_fields_are_eq(&self) {}
}
#[automatically_derived]
#[allow(nonstandard_style)]
impl ::core::clone::Clone for ThisProgram {
    #[inline]
    fn clone(&self) -> ThisProgram {
        match self {
            ThisProgram::EntryMetadata => ThisProgram::EntryMetadata,
            ThisProgram::EntryMinglingLinter => ThisProgram::EntryMinglingLinter,
            ThisProgram::ErrorDispatcherNotFound => ThisProgram::ErrorDispatcherNotFound,
            ThisProgram::ErrorRendererNotFound => ThisProgram::ErrorRendererNotFound,
            ThisProgram::ErrorSerdeJson => ThisProgram::ErrorSerdeJson,
            ThisProgram::Metadata => ThisProgram::Metadata,
            ThisProgram::ResultEmpty => ThisProgram::ResultEmpty,
            ThisProgram::ResultMetadata => ThisProgram::ResultMetadata,
        }
    }
}
impl ::std::fmt::Display for ThisProgram {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            ThisProgram::EntryMetadata => f.write_fmt(format_args!("EntryMetadata")),
            ThisProgram::EntryMinglingLinter => {
                f.write_fmt(format_args!("EntryMinglingLinter"))
            }
            ThisProgram::ErrorDispatcherNotFound => {
                f.write_fmt(format_args!("ErrorDispatcherNotFound"))
            }
            ThisProgram::ErrorRendererNotFound => {
                f.write_fmt(format_args!("ErrorRendererNotFound"))
            }
            ThisProgram::ErrorSerdeJson => f.write_fmt(format_args!("ErrorSerdeJson")),
            ThisProgram::Metadata => f.write_fmt(format_args!("Metadata")),
            ThisProgram::ResultEmpty => f.write_fmt(format_args!("ResultEmpty")),
            ThisProgram::ResultMetadata => f.write_fmt(format_args!("ResultMetadata")),
        }
    }
}
impl ::mingling::ProgramCollect for ThisProgram {
    type Enum = ThisProgram;
    type ErrorDispatcherNotFound = ErrorDispatcherNotFound;
    type ErrorRendererNotFound = ErrorRendererNotFound;
    type ResultEmpty = ResultEmpty;
    fn build_renderer_not_found(
        member_id: Self::Enum,
    ) -> ::mingling::AnyOutput<Self::Enum> {
        ::mingling::AnyOutput::new(ErrorRendererNotFound::new(member_id.to_string()))
    }
    fn build_dispatcher_not_found(
        args: Vec<String>,
    ) -> ::mingling::AnyOutput<Self::Enum> {
        ::mingling::AnyOutput::new(ErrorDispatcherNotFound::new(args))
    }
    fn build_empty_result() -> ::mingling::AnyOutput<Self::Enum> {
        ::mingling::AnyOutput::new(ResultEmpty)
    }
    fn render(any: ::mingling::AnyOutput<Self::Enum>) -> ::mingling::RenderResult {
        match any.member_id {
            Self::ErrorSerdeJson => {
                let value = unsafe {
                    any.downcast::<crate::errors::serde_json::ErrorSerdeJson>()
                        .unwrap_unchecked()
                };
                <crate::errors::serde_json::__internal_renderer_render_error_serde_json as ::mingling::Renderer>::render(
                    value,
                )
            }
            Self::Metadata => {
                let value = unsafe {
                    any.downcast::<cargo_metadata::Metadata>().unwrap_unchecked()
                };
                <crate::metadata::__internal_renderer_render_metadata as ::mingling::Renderer>::render(
                    value,
                )
            }
            _ => ::mingling::RenderResult::default(),
        }
    }
    fn do_chain(
        any: ::mingling::AnyOutput<Self::Enum>,
    ) -> ::mingling::ChainProcess<Self::Enum> {
        match any.member_id {
            Self::EntryMetadata => {
                let value = unsafe {
                    any.downcast::<crate::metadata::cmd_metadata::EntryMetadata>()
                        .unwrap_unchecked()
                };
                <crate::metadata::cmd_metadata::__internal_chain_handle_metadata as ::mingling::Chain<
                    Self::Enum,
                >>::proc(value)
            }
            Self::EntryMinglingLinter => {
                let value = unsafe {
                    any.downcast::<crate::linter::cmd_mlint::EntryMinglingLinter>()
                        .unwrap_unchecked()
                };
                <crate::linter::cmd_mlint::__internal_chain_handle_mlint as ::mingling::Chain<
                    Self::Enum,
                >>::proc(value)
            }
            _ => {
                ::core::panicking::panic_fmt(
                    format_args!("No chain found for type id: {0:?}", any.type_id),
                );
            }
        }
    }
    fn render_help(any: ::mingling::AnyOutput<Self::Enum>) -> ::mingling::RenderResult {
        #[allow(unused_imports)]
        use crate::metadata::cmd_metadata::__internal_chain_handle_metadata;
        use crate::metadata::cmd_metadata::ResultMetadata;
        use crate::linter::cmd_mlint::__internal_chain_handle_mlint;
        use crate::metadata::__internal_renderer_render_metadata;
        use crate::metadata::cmd_metadata::CMDMetadata;
        use crate::metadata::setup::ResMetadata;
        use crate::metadata::cmd_metadata::EntryMetadata;
        use crate::linter::cmd_mlint::CMDMinglingLinter;
        use crate::errors::serde_json::ErrorSerdeJson;
        use cargo_metadata::Metadata;
        use crate::errors::serde_json::__internal_renderer_render_error_serde_json;
        use crate::linter::cmd_mlint::EntryMinglingLinter;
        match any.member_id {
            _ => ::mingling::RenderResult::default(),
        }
    }
    fn has_renderer(any: &::mingling::AnyOutput<Self::Enum>) -> bool {
        match any.member_id {
            Self::ErrorSerdeJson => true,
            Self::Metadata => true,
            _ => false,
        }
    }
    fn has_chain(any: &::mingling::AnyOutput<Self::Enum>) -> bool {
        match any.member_id {
            Self::EntryMetadata => true,
            Self::EntryMinglingLinter => true,
            _ => false,
        }
    }
    fn structural_render(
        any: ::mingling::AnyOutput<Self::Enum>,
        setting: &::mingling::StructuralRendererSetting,
    ) -> Result<
        ::mingling::RenderResult,
        ::mingling::error::StructuralRendererSerializeError,
    > {
        #[allow(unused_imports)]
        use crate::metadata::cmd_metadata::__internal_chain_handle_metadata;
        use crate::metadata::cmd_metadata::ResultMetadata;
        use crate::linter::cmd_mlint::__internal_chain_handle_mlint;
        use crate::metadata::__internal_renderer_render_metadata;
        use crate::metadata::cmd_metadata::CMDMetadata;
        use crate::metadata::setup::ResMetadata;
        use crate::metadata::cmd_metadata::EntryMetadata;
        use crate::linter::cmd_mlint::CMDMinglingLinter;
        use crate::errors::serde_json::ErrorSerdeJson;
        use cargo_metadata::Metadata;
        use crate::errors::serde_json::__internal_renderer_render_error_serde_json;
        use crate::linter::cmd_mlint::EntryMinglingLinter;
        match any.member_id {
            Self::Metadata => {
                let raw = unsafe { any.restore::<Metadata>().unwrap_unchecked() };
                let mut __renderer_inner_result = ::mingling::RenderResult::default();
                ::mingling::StructuralRenderer::render(
                    &raw,
                    setting,
                    &mut __renderer_inner_result,
                )?;
                Ok(__renderer_inner_result)
            }
            _ => {
                let mut r = ::mingling::RenderResult::default();
                ::mingling::StructuralRenderer::render(&ResultEmpty, setting, &mut r)?;
                Ok(r)
            }
        }
    }
}
impl ThisProgram {
    /// Creates a new `Program<#name>` instance with default configuration.
    pub fn new() -> ::mingling::Program<ThisProgram> {
        ::mingling::Program::new()
    }
    /// Returns a static reference to the global `Program<#name>` singleton.
    pub fn this() -> &'static ::mingling::Program<ThisProgram> {
        &::mingling::this::<ThisProgram>()
    }
}
