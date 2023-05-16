use crate::error::Error;
use crate::message::Method;

#[derive(Debug, Clone, PartialEq)]
pub struct Transport {
    lower: Option<Lower>,
    parameters: Vec<Parameter>,
}

impl Transport {
    pub fn new() -> Self {
        Self {
            lower: None,
            parameters: Vec::new(),
        }
    }

    pub fn with_lower_protocol(mut self, lower: Lower) -> Self {
        self.lower = Some(lower);
        self
    }

    pub fn with_parameter(mut self, parameter: Parameter) -> Self {
        self.parameters.push(parameter);
        self
    }

    pub fn with_parameters(mut self, parameters: impl IntoIterator<Item = Parameter>) -> Self {
        self.parameters.extend(parameters);
        self
    }

    pub fn lower_protocol(&self) -> Option<&Lower> {
        self.lower.as_ref()
    }

    pub fn parameters(&self) -> &impl IntoIterator<Item = Parameter> {
        &self.parameters
    }

    pub fn parameters_iter(&self) -> impl Iterator<Item = &Parameter> {
        self.parameters.iter()
    }

    pub fn destination(&self) -> Option<&std::net::IpAddr> {
        self.parameters_iter()
            .filter_map(|parameter| {
                if let Parameter::Destination(ip_addr) = parameter {
                    Some(ip_addr)
                } else {
                    None
                }
            })
            .next()
    }

    pub fn port(&self) -> Option<&Port> {
        self.parameters_iter()
            .filter_map(|parameter| {
                if let Parameter::Port(port) = parameter {
                    Some(port)
                } else {
                    None
                }
            })
            .next()
    }

    pub fn client_port(&self) -> Option<&Port> {
        self.parameters_iter()
            .filter_map(|parameter| {
                if let Parameter::ClientPort(port) = parameter {
                    Some(port)
                } else {
                    None
                }
            })
            .next()
    }

    pub fn server_port(&self) -> Option<&Port> {
        self.parameters_iter()
            .filter_map(|parameter| {
                if let Parameter::ServerPort(port) = parameter {
                    Some(port)
                } else {
                    None
                }
            })
            .next()
    }

    pub fn interleaved_channel(&self) -> Option<&Channel> {
        self.parameters_iter()
            .filter_map(|parameter| {
                if let Parameter::Interleaved(channel) = parameter {
                    Some(channel)
                } else {
                    None
                }
            })
            .next()
    }
}

impl Default for Transport {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Transport {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "RTP/AVP")?;
        if let Some(lower) = self.lower.as_ref() {
            write!(f, "/{lower}")?;
        }
        for parameter in self.parameters.iter() {
            write!(f, ";{parameter}")?;
        }
        Ok(())
    }
}

impl std::str::FromStr for Transport {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (spec, params) = s
            .split_once(';')
            .map(|(spec, params)| (spec, Some(params)))
            .unwrap_or_else(|| (s, None));

        if spec.starts_with("RTP/AVP") {
            let lower = spec
                .split('/')
                .nth(2)
                .map(|lower| lower.parse())
                .transpose()?;

            let parameters = params
                .map(|params| {
                    params
                        .split(';')
                        .map(|p| p.parse())
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?
                .unwrap_or_default();

            Ok(Transport { lower, parameters })
        } else {
            Err(Error::TransportProtocolProfileMissing {
                value: s.to_string(),
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Lower {
    Tcp,
    Udp,
}

impl std::fmt::Display for Lower {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Lower::Tcp => write!(f, "TCP"),
            Lower::Udp => write!(f, "UDP"),
        }
    }
}

impl std::str::FromStr for Lower {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TCP" => Ok(Lower::Tcp),
            "UDP" => Ok(Lower::Udp),
            _ => Err(Error::TransportLowerUnknown {
                value: s.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Parameter {
    Unicast,
    Multicast,
    Destination(std::net::IpAddr),
    Interleaved(Channel),
    Append,
    Ttl(usize),
    Layers(usize),
    Port(Port),
    ClientPort(Port),
    ServerPort(Port),
    Ssrc(String),
    Mode(Method),
}

impl std::fmt::Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Parameter::Unicast => {
                write!(f, "unicast")
            }
            Parameter::Multicast => {
                write!(f, "multicast")
            }
            Parameter::Destination(host) => {
                write!(f, "destination={host}")
            }
            Parameter::Interleaved(channel) => {
                write!(f, "interleaved={channel}")
            }
            Parameter::Append => {
                write!(f, "append")
            }
            Parameter::Ttl(ttl) => {
                write!(f, "ttl={ttl}")
            }
            Parameter::Layers(layers) => {
                write!(f, "layers={layers}")
            }
            Parameter::Port(port) => {
                write!(f, "port={port}")
            }
            Parameter::ClientPort(client_port) => {
                write!(f, "client_port={client_port}")
            }
            Parameter::ServerPort(server_port) => {
                write!(f, "server_port={server_port}")
            }
            Parameter::Ssrc(ssrc) => {
                write!(f, "ssrc={ssrc}")
            }
            Parameter::Mode(method) => {
                write!(f, "mode=\"{method}\"")
            }
        }
    }
}

impl std::str::FromStr for Parameter {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('=');
        let var = parts
            .next()
            .ok_or_else(|| Error::TransportParameterInvalid {
                parameter: s.to_string(),
            })?;

        let mut val_or_err = || {
            parts
                .next()
                .ok_or_else(|| Error::TransportParameterValueMissing {
                    var: var.to_string(),
                })
        };

        fn parse_or_err<T: std::str::FromStr>(var: &str, val: &str) -> Result<T, Error> {
            val.parse::<T>()
                .map_err(|_| Error::TransportParameterValueInvalid {
                    var: var.to_string(),
                    val: val.to_string(),
                })
        }

        match var {
            "unicast" => Ok(Parameter::Unicast),
            "multicast" => Ok(Parameter::Multicast),
            "destination" => {
                let val = val_or_err()?;
                let host = parse_or_err(var, val)?;
                Ok(Parameter::Destination(host))
            }
            "interleaved" => {
                let val = val_or_err()?;
                let channel = parse_or_err(var, val)?;
                Ok(Parameter::Interleaved(channel))
            }
            "append" => Ok(Parameter::Append),
            "ttl" => {
                let val = val_or_err()?;
                let ttl = parse_or_err(var, val)?;
                Ok(Parameter::Ttl(ttl))
            }
            "layers" => {
                let val = val_or_err()?;
                let layers = parse_or_err(var, val)?;
                Ok(Parameter::Layers(layers))
            }
            "port" => {
                let val = val_or_err()?;
                let port = parse_or_err(var, val)?;
                Ok(Parameter::Port(port))
            }
            "client_port" => {
                let val = val_or_err()?;
                let port = parse_or_err(var, val)?;
                Ok(Parameter::ClientPort(port))
            }
            "server_port" => {
                let val = val_or_err()?;
                let port = parse_or_err(var, val)?;
                Ok(Parameter::ServerPort(port))
            }
            "ssrc" => {
                let val = val_or_err()?;
                Ok(Parameter::Ssrc(val.to_string()))
            }
            "mode" => {
                let val = val_or_err()?;
                let val = val
                    .strip_prefix('"')
                    .unwrap_or(val)
                    .strip_suffix('"')
                    .unwrap_or(val);
                let method = parse_or_err(var, val)?;
                Ok(Parameter::Mode(method))
            }
            _ => Err(Error::TransportParameterUnknown {
                var: var.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Channel {
    Single(u8),
    Range(u8, u8),
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Channel::Single(channel) => {
                write!(f, "{channel}")
            }
            Channel::Range(channel_1, channel_2) => {
                write!(f, "{channel_1}-{channel_2}")
            }
        }
    }
}

impl std::str::FromStr for Channel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('-');
        let channel_1 = parts
            .next()
            .and_then(|channel| channel.parse::<u8>().ok())
            .ok_or_else(|| Error::TransportChannelMalformed {
                value: s.to_string(),
            })?;
        let channel_2 = parts.next().map(|channel| {
            channel
                .parse::<u8>()
                .map_err(|_| Error::TransportChannelMalformed {
                    value: s.to_string(),
                })
        });

        Ok(if let Some(channel_2) = channel_2 {
            Channel::Range(channel_1, channel_2?)
        } else {
            Channel::Single(channel_1)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Port {
    Single(u16),
    Range(u16, u16),
}

impl std::fmt::Display for Port {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Port::Single(port) => {
                write!(f, "{port}")
            }
            Port::Range(port_1, port_2) => {
                write!(f, "{port_1}-{port_2}")
            }
        }
    }
}

impl std::str::FromStr for Port {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('-');
        let port_1 = parts
            .next()
            .and_then(|port| port.parse::<u16>().ok())
            .ok_or_else(|| Error::TransportPortMalformed {
                value: s.to_string(),
            })?;
        let port_2 = parts.next().map(|port| {
            port.parse::<u16>()
                .map_err(|_| Error::TransportPortMalformed {
                    value: s.to_string(),
                })
        });

        Ok(if let Some(port_2) = port_2 {
            Port::Range(port_1, port_2?)
        } else {
            Port::Single(port_1)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal() {
        assert_eq!("RTP/AVP".parse::<Transport>().unwrap(), Transport::new(),);
    }

    #[test]
    fn parse_lower_tcp() {
        assert_eq!(
            "RTP/AVP/TCP".parse::<Transport>().unwrap(),
            Transport::new().with_lower_protocol(Lower::Tcp),
        );
    }

    #[test]
    fn parse_lower_udp() {
        assert_eq!(
            "RTP/AVP/UDP".parse::<Transport>().unwrap(),
            Transport::new().with_lower_protocol(Lower::Udp),
        );
    }

    #[test]
    fn parse_unicast() {
        assert_eq!(
            "RTP/AVP;unicast".parse::<Transport>().unwrap(),
            Transport::new().with_parameter(Parameter::Unicast),
        );
    }

    #[test]
    fn parse_destination_missing_value() {
        assert!(matches!(
            "RTP/AVP/UDP;destination".parse::<Transport>(),
            Err(Error::TransportParameterValueMissing { var: _ }),
        ),);
    }

    #[test]
    fn parse_destination_ip() {
        assert_eq!(
            "RTP/AVP/UDP;destination=127.0.0.1"
                .parse::<Transport>()
                .unwrap(),
            Transport::new()
                .with_lower_protocol(Lower::Udp)
                .with_parameter(Parameter::Destination([127, 0, 0, 1].into())),
        );
    }

    #[test]
    fn parse_interleaved_invalid() {
        assert!(matches!(
            "RTP/AVP/UDP;interleaved=invalid".parse::<Transport>(),
            Err(Error::TransportParameterValueInvalid { var: _, val: _ }),
        ),);
    }

    #[test]
    fn parse_interleaved_channel() {
        assert_eq!(
            "RTP/AVP/UDP;interleaved=8-9".parse::<Transport>().unwrap(),
            Transport::new()
                .with_lower_protocol(Lower::Udp)
                .with_parameter(Parameter::Interleaved(Channel::Range(8, 9))),
        );
    }

    #[test]
    fn parse_layers() {
        assert_eq!(
            "RTP/AVP/UDP;layers=3".parse::<Transport>().unwrap(),
            Transport::new()
                .with_lower_protocol(Lower::Udp)
                .with_parameter(Parameter::Layers(3)),
        );
    }

    #[test]
    fn parse_port_single() {
        assert_eq!(
            "RTP/AVP/UDP;port=3".parse::<Transport>().unwrap(),
            Transport::new()
                .with_lower_protocol(Lower::Udp)
                .with_parameter(Parameter::Port(Port::Single(3))),
        );
    }

    #[test]
    fn parse_server_port_range() {
        assert_eq!(
            "RTP/AVP/UDP;server_port=3-4".parse::<Transport>().unwrap(),
            Transport::new()
                .with_lower_protocol(Lower::Udp)
                .with_parameter(Parameter::ServerPort(Port::Range(3, 4))),
        );
    }

    #[test]
    fn parse_ssrc() {
        assert_eq!(
            "RTP/AVP/UDP;ssrc=ABCDEF".parse::<Transport>().unwrap(),
            Transport::new()
                .with_lower_protocol(Lower::Udp)
                .with_parameter(Parameter::Ssrc("ABCDEF".to_string())),
        );
    }

    #[test]
    fn parse_mode_method_unknown() {
        assert!(matches!(
            "RTP/AVP/UDP;mode=UNKNOWN".parse::<Transport>(),
            Err(Error::TransportParameterValueInvalid { var: _, val: _ }),
        ),);
    }

    #[test]
    fn parse_mode_method() {
        assert_eq!(
            "RTP/AVP/UDP;mode=PLAY".parse::<Transport>().unwrap(),
            Transport::new()
                .with_lower_protocol(Lower::Udp)
                .with_parameter(Parameter::Mode(Method::Play)),
        );
        assert_eq!(
            "RTP/AVP/UDP;mode=\"PLAY\"".parse::<Transport>().unwrap(),
            Transport::new()
                .with_lower_protocol(Lower::Udp)
                .with_parameter(Parameter::Mode(Method::Play)),
        );
    }

    #[test]
    fn parse_rfc2326_section_12_39_examples() {
        assert_eq!(
            "RTP/AVP;multicast;ttl=127;mode=\"PLAY\""
                .parse::<Transport>()
                .unwrap(),
            Transport::new()
                .with_parameter(Parameter::Multicast)
                .with_parameter(Parameter::Ttl(127))
                .with_parameter(Parameter::Mode(Method::Play)),
        );
        assert_eq!(
            "RTP/AVP;unicast;client_port=3456-3457;mode=\"PLAY\""
                .parse::<Transport>()
                .unwrap(),
            Transport::new()
                .with_parameter(Parameter::Unicast)
                .with_parameter(Parameter::ClientPort(Port::Range(3456, 3457)))
                .with_parameter(Parameter::Mode(Method::Play)),
        );
    }

    #[test]
    fn format_minimal() {
        assert_eq!(&Transport::new().to_string(), "RTP/AVP",);
    }

    #[test]
    fn format_lower_tcp() {
        assert_eq!(
            &Transport::new().with_lower_protocol(Lower::Tcp).to_string(),
            "RTP/AVP/TCP",
        );
    }

    #[test]
    fn format_lower_udp() {
        assert_eq!(
            &Transport::new().with_lower_protocol(Lower::Udp).to_string(),
            "RTP/AVP/UDP",
        );
    }

    #[test]
    fn format_unicast() {
        assert_eq!(
            &Transport::new()
                .with_lower_protocol(Lower::Udp)
                .with_parameter(Parameter::Unicast)
                .to_string(),
            "RTP/AVP/UDP;unicast",
        );
    }

    #[test]
    fn format_rfc2326_section_12_39_examples() {
        assert_eq!(
            &Transport::new()
                .with_parameter(Parameter::Multicast)
                .with_parameter(Parameter::Ttl(127))
                .with_parameter(Parameter::Mode(Method::Play))
                .to_string(),
            "RTP/AVP;multicast;ttl=127;mode=\"PLAY\"",
        );
        assert_eq!(
            &Transport::new()
                .with_parameter(Parameter::Unicast)
                .with_parameter(Parameter::ClientPort(Port::Range(3456, 3457)))
                .with_parameter(Parameter::Mode(Method::Play))
                .to_string(),
            "RTP/AVP;unicast;client_port=3456-3457;mode=\"PLAY\"",
        );
    }

    #[test]
    fn format_all_parameters() {
        assert_eq!(
            &Transport::new()
                .with_lower_protocol(Lower::Tcp)
                .with_parameter(Parameter::Unicast)
                .with_parameter(Parameter::Multicast)
                .with_parameter(Parameter::Destination([1, 2, 3, 4].into()))
                .with_parameter(Parameter::Interleaved(Channel::Range(12, 13)))
                .with_parameter(Parameter::Append)
                .with_parameter(Parameter::Ttl(999))
                .with_parameter(Parameter::Layers(2))
                .with_parameter(Parameter::Port(Port::Single(8)))
                .with_parameter(Parameter::ClientPort(Port::Range(9, 10)))
                .with_parameter(Parameter::ServerPort(Port::Range(11, 12)))
                .with_parameter(Parameter::Ssrc("01234ABCDEF".to_string()))
                .with_parameter(Parameter::Mode(Method::Describe))
                .to_string(),
            "RTP/AVP/TCP;unicast;multicast;destination=1.2.3.4;interleaved=1234-1235;\
                append;ttl=999;layers=2;port=8;client_port=9-10;server_port=11-12;\
                ssrc=01234ABCDEF;mode=\"DESCRIBE\"",
        );
    }
}
