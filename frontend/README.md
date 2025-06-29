# Fathom to Loom Frontend

A modern Dioxus-based frontend application for the Fathom to Loom service.

## Features

### Pages & Components

- **Authentication** (login/register forms) with validation
- **Dashboard** - Real-time queue status, global position tracking, worker progress bars
- **Recordings** - Browse Fathom meetings with "Add to queue" functionality  
- **Settings** - Encrypted API key management and user preferences

### Technical Features

- **Real-time Updates** - WebSocket integration for live queue and worker status
- **Responsive Design** - Tailwind CSS for modern, mobile-first styling
- **Authentication** - Token-based auth with secure local storage
- **Error Handling** - Comprehensive error states and validation
- **Loading States** - Smooth loading indicators and skeleton screens

## Architecture

### Project Structure

```
src/
├── components/          # Reusable UI components
│   ├── auth.rs         # Authentication components
│   ├── common.rs       # Common UI elements
│   ├── dashboard.rs    # Dashboard-specific components
│   └── layout.rs       # App layout and navigation
├── pages/              # Route pages
│   ├── auth.rs         # Login/Register pages
│   ├── dashboard.rs    # Dashboard page
│   ├── home.rs         # Landing page
│   ├── recordings.rs   # Recordings list page
│   └── settings.rs     # Settings page
├── services/           # Business logic services
│   ├── api.rs          # REST API client
│   ├── auth.rs         # Authentication service
│   └── websocket.rs    # WebSocket client
├── utils/              # Utility functions
│   ├── date.rs         # Date formatting
│   └── validation.rs   # Input validation
├── config.rs           # Configuration management
└── main.rs             # App entry point
```

### Technologies Used

- **Dioxus 0.5** - Rust-based React-like framework
- **dioxus-router** - Client-side routing
- **dioxus-liveview** - Real-time updates (alternative to WebSocket)
- **gloo-net** - HTTP client for WASM
- **ws_stream_wasm** - WebSocket support
- **Tailwind CSS** - Utility-first CSS framework
- **validator** - Form validation
- **chrono** - Date/time handling

## API Integration

### REST Endpoints

- `GET/POST /api/queue` - Queue management
- `GET /api/meetings` - Fathom recordings
- `GET/PUT /api/keys` - Encrypted API keys
- `POST /auth/login` - Authentication
- `POST /auth/register` - User registration

### WebSocket Events

- Queue updates with real-time position tracking
- Worker status with progress indicators
- Connection health monitoring

### Authentication

- JWT token-based authentication
- Secure token storage in localStorage
- Automatic token refresh
- Protected route guards

## Real-time Features

### Dashboard

- Live queue position updates
- Worker progress bars
- Estimated wait times
- Connection status indicator

### WebSocket Integration

- Automatic reconnection with exponential backoff
- Connection health monitoring
- Message type safety with Rust enums
- Graceful degradation for connectivity issues

## Styling

### Tailwind CSS

- Modern utility-first styling
- Responsive design patterns
- Custom color palette (indigo-based)
- Smooth animations and transitions

### Component Design

- Consistent spacing and typography
- Loading states and skeletons
- Error and success message patterns
- Interactive hover and focus states

## Development

### Building

```bash
# Development build
dx serve

# Production build  
dx build --release

# With Tailwind CSS
dx build --features tailwind
```

### Testing

```bash
# Run tests
cargo test --target wasm32-unknown-unknown

# Run clippy
cargo clippy --target wasm32-unknown-unknown
```

### Environment Variables

- `API_BASE_URL` - Backend API base URL
- `ENVIRONMENT` - Environment (development/production)

## Security

- XSS protection via Dioxus's type-safe templates
- CSRF protection through SameSite cookies
- Encrypted API key storage
- Secure token management
- Input validation and sanitization

## Performance

- Lazy loading for large meeting lists
- Efficient WebSocket message handling
- Minimal bundle size with Rust/WASM
- Optimized re-renders with Dioxus signals
- CDN-delivered Tailwind CSS

## Browser Support

- Modern browsers with WebAssembly support
- Progressive enhancement for WebSocket features
- Responsive design for mobile devices
