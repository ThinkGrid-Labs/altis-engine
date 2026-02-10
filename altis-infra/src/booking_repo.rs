use sqlx::Postgres;
use uuid::Uuid;
use altis_domain::booking::{Booking, Passenger, BookingStatus};
use chrono::Utc;

pub struct BookingRepository;

impl BookingRepository {
    pub async fn create_booking(
        tx: &mut sqlx::Transaction<'_, Postgres>,
        booking_id: Uuid,
        flight_id: Uuid,
        user_email: &str,
        price_amount: i32,
        price_currency: &str,
    ) -> Result<Booking, sqlx::Error> {
        let now = Utc::now();
        let status = BookingStatus::PENDING.to_string();

        let booking = Booking {
            id: booking_id,
            flight_id,
            user_email: user_email.to_string(),
            status: BookingStatus::PENDING,
            total_price_amount: price_amount,
            total_price_currency: price_currency.to_string(),
            created_at: now,
            updated_at: now,
        };

        sqlx::query!(
            r#"
            INSERT INTO bookings (id, flight_id, user_email, status, total_price_amount, total_price_currency, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            booking.id,
            booking.flight_id,
            booking.user_email,
            status,
            booking.total_price_amount,
            booking.total_price_currency,
            booking.created_at,
            booking.updated_at
        )
        .execute(&mut **tx)
        .await?;

        Ok(booking)
    }

    pub async fn add_passenger(
        tx: &mut sqlx::Transaction<'_, Postgres>,
        booking_id: Uuid,
        first_name: &str,
        last_name: &str,
        seats: &Vec<crate::booking::PassengerSeat>, // altis_domain::booking::PassengerSeat
    ) -> Result<Passenger, sqlx::Error> {
        let passenger_id = Uuid::new_v4();
        
        // Insert Passenger
        sqlx::query!(
            r#"
            INSERT INTO passengers (id, booking_id, first_name, last_name, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            passenger_id,
            booking_id,
            first_name,
            last_name,
            Utc::now()
        )
        .execute(&mut **tx)
        .await?;

        // Insert Seats
        for seat in seats {
            sqlx::query!(
                r#"
                INSERT INTO passenger_seats (passenger_id, flight_id, seat_number)
                VALUES ($1, $2, $3)
                "#,
                passenger_id,
                seat.flight_id,
                seat.seat_number
            )
            .execute(&mut **tx)
            .await?;
        }

        Ok(Passenger {
            id: passenger_id,
            booking_id,
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            date_of_birth: None,
            seats: vec![], // Populate if needed
        })
    }

    pub async fn confirm_booking(
        tx: &mut sqlx::Transaction<'_, Postgres>,
        booking_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let status = BookingStatus::CONFIRMED.to_string();
        sqlx::query!(
            r#"
            UPDATE bookings SET status = $1, updated_at = $2 WHERE id = $3
            "#,
            status,
            Utc::now(),
            booking_id
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }
}
