#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trip_serialization() {
        let claims = Claims { sub: "user-1".to_string(), role: "guest".to_string(), exp: 0 };
        let flight_1 = Uuid::new_v4();
        let flight_2 = Uuid::new_v4();
        
        let flight_ids = vec![flight_1, flight_2];
        let flight_ids_str = flight_ids.iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(",");
            
        let value = format!("{}|{}", flight_ids_str, claims.sub);
        
        assert_eq!(value, format!("{},{}|user-1", flight_1, flight_2));
        
        // Parse back
        let parts: Vec<&str> = value.split('|').collect();
        assert_eq!(parts.len(), 2);
        
        let flights: Vec<&str> = parts[0].split(',').collect();
        assert_eq!(flights.len(), 2);
        assert_eq!(flights[0], flight_1.to_string());
    }
}
